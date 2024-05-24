use anyhow::{anyhow, Result};
use memmap::Mmap;
use std::fs::File;
use std::simd::prelude::*;

pub const MARKER_BYTES: usize = 2;

#[derive(Debug, PartialEq)]
pub struct MarLen {
    offset: usize,
    length: usize,
}

/// JpegReader parses through the mmap, validates markers and prepares data for decoding
pub struct JpegReader {
    pub mmap: Mmap,
    pub cursor: usize,
}

impl JpegReader {
    pub fn from_file(file: File) -> Result<Self> {
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(JpegReader { mmap, cursor: 0 })
    }

    pub fn from_file_path(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        JpegReader::from_file(file)
    }

    fn at_eof(&self) -> bool {
        self.cursor >= self.mmap.len()
    }

    fn within_bound(&self, seek_by: usize) -> bool {
        // 0 <= self.cursor, self.cursor + seek_by < self.mmap.len()
        self.cursor >= 0 && self.cursor + seek_by < self.mmap.len()
    }

    fn parse_marlen(&mut self, expected_markers: Simd<u8, 2>) -> Result<MarLen> {
        let marker = u8x2::from_slice(&self.mmap[self.cursor..self.cursor + MARKER_BYTES]);
        if !marker.simd_eq(expected_markers).all() {
            return Err(anyhow!("expected markers and markers found do not align."));
        }
        self.cursor += MARKER_BYTES;

        let length = u16::from_be_bytes([self.mmap[self.cursor], self.mmap[self.cursor + 1]]);
        self.cursor += MARKER_BYTES;

        return Ok(MarLen {
            offset: self.cursor,
            length: length as usize,
        });
    }

    fn check_prelude(&mut self) -> Result<()> {
        // The JPEG File Interchange Format requires the APP0 marker right after the SOI marker.
        let markers = u8x4::from_slice(&self.mmap[self.cursor..self.cursor + (MARKER_BYTES * 2)]);
        self.cursor += 4;

        let expected_markers = u8x4::from_array([0xFF, 0xD8, 0xFF, 0xE0]);
        let mask_markers = markers.simd_eq(expected_markers);

        match mask_markers.all() {
            true => Ok(()),
            false => Err(anyhow!("Expected the SOI marker and APP0 marker.")),
        }
    }

    fn check_postlude(&mut self) -> Result<()> {
        let eoi_marker = u8x2::from_slice(&self.mmap[self.mmap.len() - MARKER_BYTES..]);
        let expected = u8x2::from_array([0xFF, 0xD9]);

        match eoi_marker.simd_eq(expected).all() {
            true => Ok(()),
            false => Err(anyhow!(
                "Expected the EOI marker to appear as the last two bytes in image data"
            )),
        }
    }

    fn parse_headers(&mut self) -> Result<()> {
        if !self.within_bound(MARKER_BYTES) {
            return Err(anyhow!(
                "we've reached the eof, unable to parse header length"
            ));
        }

        let length =
            u16::from_be_bytes([self.mmap[self.cursor], self.mmap[self.cursor + 1]]) as usize;
        self.cursor += 2;

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

        self.cursor += length;

        Ok(())
    }

    pub fn find_huffman_markers(&mut self) -> Result<Vec<MarLen>> {
        let simd_size = 64;
        let mut temp_chunk = [0u8; 64];

        let mut huffman_marlens = vec![];

        while self.cursor < self.mmap.len() - 2 {
            let end = (self.cursor + simd_size).min(self.mmap.len() - 2);
            let len = end - self.cursor;

            temp_chunk[..len].copy_from_slice(&self.mmap[self.cursor..end]);
            let simd_chunk = u8x64::from_array(temp_chunk);
            // simd_chunk: [0x00, 0x00, 0xFF, 0xC4, 0x00, 0xFF, 0x00, 0xC4]

            let mask_ff = u8x64::splat(0xFF);
            // mask_ff:    [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF]

            let mask_c4 = u8x64::splat(0xC4);
            // mask_c4:    [0xC4, 0xC4, 0xC4, 0xC4, 0xC4, 0xC4, 0xC4, 0xC4]

            let ff_matches = simd_chunk.simd_eq(mask_ff);
            // ff_matches: [false, false, true, false, false, true, false, false]

            if !ff_matches.any() {
                self.cursor += simd_size;
                continue
            }

            let next_byte_chunk = simd_chunk.rotate_elements_left::<1>();
            // next_byte_chunk: [0x00, 0xFF, 0xC4, 0x00, 0xFF, 0x00, 0xC4, 0x00]

            let c4_matches = next_byte_chunk.simd_eq(mask_c4);
            // c4_matches : [false, false, true, false, false, false, true, false]

            let mut ffc4_mask = ff_matches & c4_matches;
            // ffc4_mask: [false, false, true, false, false, true*, false, false] <-- * suppose we had another true

            let curr_iter_index = self.cursor;
            while let Some(huffman_marker_index) = ffc4_mask.first_set() {
                ffc4_mask.set(huffman_marker_index, false);
                // ffc4_mask: [false, false, false, false, false, true, false, false]
                self.cursor += huffman_marker_index;

                let huffman_marlen= self.parse_marlen(Simd::from_array([0xFF, 0xC4]))?;
                huffman_marlens.push(huffman_marlen);

                self.cursor = curr_iter_index;
            }

            self.cursor += simd_size
        }


        Ok(huffman_marlens)
    }


    pub fn parse(&mut self) -> Result<()> {
        self.check_prelude()?;
        self.parse_headers()?;
        self.check_postlude()?;

        let _huffman_tables = self.find_huffman_markers()?;


        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;
    use std::io::Write;
    use std::sync::Once;
    use super::*;
    #[test]
    fn test_parse() -> Result<()> {
        let mut jpeg_reader= JpegReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.parse().is_ok());
        Ok(())
    }

    #[test]
    fn test_check_prelude() -> Result<()> {
        let mut jpeg_reader = JpegReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.check_prelude().is_ok());
        Ok(())
    }

    #[test]
    fn test_check_postlude() -> Result<()> {
        let mut jpeg_reader = JpegReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.check_postlude().is_ok());
        Ok(())
    }

    #[test]
    fn test_find_huffman_markers() -> Result<()> {
        let mut jpeg_reader = JpegReader::from_file_path("mike.jpg")?;

        let huffman_markers = jpeg_reader.find_huffman_markers();
        assert!(huffman_markers.is_ok());

        let huffman_markers = huffman_markers.unwrap();
        assert_eq!(4, huffman_markers.len());
        println!("huffman_markers: {:?}", huffman_markers);

        Ok(())
    }

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            // Create a temporary file for testing
            let data = vec![
                0x00, 0x00, 0x00, 0x00, 0xFF, 0xC4, 0x00, 0x02, b'h', b'i', 0xFF, 0xC4, 0x00, 0x03, // 13th (0-indexed)
                b'w', b'E', b'F', 0xFF, 0xC3, 0xFF, 0xFF, 0xFF, 0xFF, 0xC4, 0x00, 0x01, b'd',
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

        let mut jpeg_reader = JpegReader { mmap, cursor: 0 };

        let huffman_markers = jpeg_reader.find_huffman_markers()?;
        assert_eq!(
            huffman_markers,
            vec![
                MarLen { offset: 8, length: 2 },
                MarLen { offset: 14, length: 3 },
                MarLen { offset: 26, length: 1 }
            ]
        );

        Ok(())
    }
}
