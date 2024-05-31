use crate::jpeg_decoder::JpegDecoder;
use crate::marker::Marker;
use anyhow::{anyhow, Result};
use memmap::Mmap;
use std::fs::File;
use std::simd::prelude::*;


// Every mod length denotes a new section of data to process.
#[derive(Debug, PartialEq, Copy, Clone)]
pub struct MarLen {
    pub offset: usize,
    pub length: usize,
}

/// JFIFReader parses through the mmap, validates markers and prepares data for decoding
pub struct JFIFReader {
    pub mmap: Mmap,
    pub cursor: usize,
}

impl JFIFReader {
    pub fn from_file(file: File) -> Result<Self> {
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(JFIFReader { mmap, cursor: 0 })
    }

    pub fn from_file_path(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        JFIFReader::from_file(file)
    }

    fn at_eof(&self) -> bool {
        self.cursor >= self.mmap.len()
    }

    fn within_bound(&self, seek_by: usize) -> bool {
        // 0 <= self.cursor, self.cursor + seek_by < self.mmap.len()
        self.cursor + seek_by < self.mmap.len()
    }

    fn parse_marlen(&mut self, expected_markers: Simd<u8, 2>) -> Result<MarLen> {
        if !self.within_bound(Marker::SIZE) {
            return Err(anyhow!("out of bounds cursor: {}", self.cursor));
        }

        let marker = u8x2::from_slice(&self.mmap[self.cursor..self.cursor + Marker::SIZE]);
        if !marker.simd_eq(expected_markers).all() {
            return Err(anyhow!("expected markers and markers found do not align."));
        }
        self.cursor += Marker::SIZE;

        let length = u16::from_be_bytes([self.mmap[self.cursor], self.mmap[self.cursor + 1]]);
        self.cursor += Marker::SIZE;

        return Ok(MarLen {
            offset: self.cursor,
            length: length as usize - Marker::SIZE,
        });
    }

    fn check_prelude(&mut self) -> Result<()> {
        // The JPEG File Interchange Format requires the APP0 mod right after the SOI mod.
        let markers = u8x4::from_slice(&self.mmap[self.cursor..self.cursor + (Marker::SIZE * 2)]);
        self.cursor += Marker::SIZE * 2;

        let expected_markers = u8x4::from_array([0xFF, 0xD8, 0xFF, 0xE0]);
        let mask_markers = markers.simd_eq(expected_markers);

        match mask_markers.all() {
            true => Ok(()),
            false => Err(anyhow!("Expected the SOI mod and APP0 mod.")),
        }
    }

    fn check_postlude(&mut self) -> Result<()> {
        let eoi_marker = u8x2::from_slice(&self.mmap[self.mmap.len() - Marker::SIZE..]);
        let expected = u8x2::from_array([0xFF, 0xD9]);

        match eoi_marker.simd_eq(expected).all() {
            true => Ok(()),
            false => Err(anyhow!(
                "Expected the EOI mod to appear as the last two bytes in image data"
            )),
        }
    }

    fn parse_headers(&mut self) -> Result<()> {
        if !self.within_bound(Marker::SIZE) {
            return Err(anyhow!(
                "we've reached the eof, unable to parse header length"
            ));
        }

        let length =
            u16::from_be_bytes([self.mmap[self.cursor], self.mmap[self.cursor + 1]]) as usize;
        self.cursor += Marker::SIZE;

        if !self.within_bound(length) {
            return Err(anyhow!("we've reached the eof, unable to seek past length"));
        }

        // APP0 headers are variable
        let mut temp_array = [0u8; 8];
        let identifier_slice = &self.mmap[self.cursor..self.cursor + 5];
        temp_array[..identifier_slice.len()].copy_from_slice(identifier_slice);

        let identifier = u8x8::from_array(temp_array);
        let expected_identifier = u8x8::from([b'J', b'F', b'I', b'F', 0x00, 0, 0, 0]);

        if !identifier.simd_eq(expected_identifier).all() {
            return Err(anyhow!("identifier was not equal to expected"));
        }

        self.cursor += length - Marker::SIZE;

        Ok(())
    }

    fn find_markers(&mut self, expected: Simd<u8, 2>) -> Result<Vec<MarLen>> {
        const LANE_COUNT: usize = 64;

        let mut marlens = vec![];

        while self.cursor < self.mmap.len() - Marker::SIZE {
            let end = (self.cursor + LANE_COUNT).min(self.mmap.len() - Marker::SIZE);
            let len = end - self.cursor;

            let mut temp_chunk = [0u8; LANE_COUNT];
            temp_chunk[..len].copy_from_slice(&self.mmap[self.cursor..end]);
            let simd_chunk = u8x64::from_array(temp_chunk);

            let mask_0 = u8x64::splat(expected[0]);
            let matches_0 = simd_chunk.simd_eq(mask_0);

            if !matches_0.any() {
                self.cursor += LANE_COUNT;
                continue;
            }

            let next_byte_chunk = simd_chunk.rotate_elements_left::<1>();

            let mask_1 = u8x64::splat(expected[1]);
            let matches_1 = next_byte_chunk.simd_eq(mask_1);

            let mut matches_mask = matches_0 & matches_1;

            let curr_iter_index = self.cursor;
            while let Some(marker_index) = matches_mask.first_set() {
                matches_mask.set(marker_index, false);
                self.cursor += marker_index;

                let marlen = self.parse_marlen(expected)?;
                marlens.push(marlen);

                self.cursor = curr_iter_index;
            }

            self.cursor += LANE_COUNT
        }

        Ok(marlens)
    }

    pub(crate) fn find_huffman_markers(&mut self) -> Result<Vec<MarLen>> {
        self.find_markers(Simd::from_array([0xFF, 0xC4]))
    }

    pub(crate) fn find_dqt_markers(&mut self) -> Result<Vec<MarLen>> {
        self.find_markers(Simd::from_array([0xFF, 0xDB]))
    }

    pub(crate) fn find_sos_marker(&mut self) -> Result<MarLen> {
        let marlens = self.find_markers(Simd::from_array([0xFF, 0xDA]))?;
        debug_assert_eq!(marlens.len(), 1);

        Ok(marlens[0])
    }

    pub(crate) fn find_sof_marker(&mut self) -> Result<MarLen> {
        let marlens = self.find_markers(Simd::from_array([0xFF, 0xC0]))?;
        debug_assert_eq!(marlens.len(), 1);
        Ok(marlens[0])
    }

    pub fn decoder(&mut self) -> Result<JpegDecoder> {
        self.check_prelude()?;
        self.parse_headers()?;
        let post_header_index = self.cursor;
        self.check_postlude()?;

        // todo refactor, we can do all of this in one pass!
        let huffman_marlens = self.find_huffman_markers()?;
        self.cursor = post_header_index;

        let qt_marlens = self.find_dqt_markers()?;
        self.cursor = post_header_index;

        let sos_marlen = self.find_sos_marker()?;
        self.cursor = post_header_index;

        let sof_marlen = self.find_sof_marker()?;

        Ok(JpegDecoder::new(
            &self.mmap,
            huffman_marlens,
            qt_marlens,
            sos_marlen,
            sof_marlen,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::sync::Once;
    #[test]
    fn test_decoder() -> Result<()> {
        let mut jpeg_reader = JFIFReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.decoder().is_ok());
        Ok(())
    }

    #[test]
    fn test_check_prelude() -> Result<()> {
        let mut jpeg_reader = JFIFReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.check_prelude().is_ok());
        Ok(())
    }

    #[test]
    fn test_check_postlude() -> Result<()> {
        let mut jpeg_reader = JFIFReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.check_postlude().is_ok());
        Ok(())
    }

    #[test]
    fn test_find_huffman_markers() -> Result<()> {
        let mut jpeg_reader = JFIFReader::from_file_path("mike.jpg")?;

        let huffman_markers = jpeg_reader.find_huffman_markers();
        assert!(huffman_markers.is_ok());

        let huffman_markers = huffman_markers.unwrap();
        assert_eq!(4, huffman_markers.len());
        println!("huffman_markers: {:?}", huffman_markers);

        Ok(())
    }

    #[test]
    fn test_find_dqt_markers() -> Result<()> {
        let mut jpeg_reader = JFIFReader::from_file_path("mike.jpg")?;

        let dqt_markers = jpeg_reader.find_dqt_markers();
        assert!(dqt_markers.is_ok());

        let dqt_markers = dqt_markers.unwrap();
        assert_eq!(dqt_markers.len(), 2);
        println!("dqt markers: {:?}", dqt_markers);

        Ok(())
    }

    #[test]
    fn test_find_sos_marker() -> Result<()> {
        let mut jpeg_reader = JFIFReader::from_file_path("mike.jpg")?;
        let sos_marker = jpeg_reader.find_sos_marker();
        assert!(sos_marker.is_ok());
        let sos_marker = sos_marker.unwrap();

        println!("sos data is: {}, {}", sos_marker.offset, sos_marker.length);

        Ok(())
    }

    #[test]
    fn test_find_sof_marker() -> Result<()> {
        let mut jpeg_reader = JFIFReader::from_file_path("mike.jpg")?;
        let sof_marker = jpeg_reader.find_sof_marker();
        assert!(sof_marker.is_ok());
        let sof_marker = sof_marker.unwrap();

        assert_eq!(sof_marker.length, 17 - Marker::SIZE);
        println!("sos data is: {}, {}", sof_marker.offset, sof_marker.length);

        Ok(())
    }

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Create a temporary file for testing
            let data = vec![
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xC4, 0x00, 0x04, b'h', b'i', 0xFF, 0xC4, // 11
                0x00, 0x05, b'w', b'E', b'F', 0xFF, 0xC3, 0xFF, 0xFF, 0xFF, 0xFF, 0xC4, 0x00, 0x03,
                b'd',
            ];

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("mock_jpeg_data.bin")
                .unwrap();
            file.write_all(&data).unwrap();
        });
    }

    #[test]
    fn test_huffman_markers_basic_1() -> Result<()> {
        setup();

        let file = File::open("mock_jpeg_data.bin")?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut jpeg_reader = JFIFReader { mmap, cursor: 0 };

        let huffman_markers = jpeg_reader.find_huffman_markers()?;
        assert_eq!(
            huffman_markers,
            vec![
                MarLen {
                    offset: 8,
                    length: 2
                },
                MarLen {
                    offset: 14,
                    length: 3
                },
                MarLen {
                    offset: 26,
                    length: 1
                }
            ]
        );

        Ok(())
    }
}
