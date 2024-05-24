use anyhow::{anyhow, Result};
use memmap::Mmap;
use std::fs::File;
use std::simd::prelude::*;
use crate::jpeg_decoder::JsonDecoder;

/// JpegReader reads through the mmap, validates markers and prepares data for decoding
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
        self.cursor + seek_by <= self.mmap.len()
    }

    fn marlen(&mut self, expected_markers: Simd<u8, 2>) -> Result<usize> {
        let marker = u8x2::from_slice(&self.mmap[self.cursor..self.cursor + 2]);
        self.cursor += 2;

        if !marker.simd_eq(expected_markers).all() {
            return Err(anyhow!("expected markers and markers found do not align."));
        }

        let length = u16::from_be_bytes([self.mmap[self.cursor], self.mmap[self.cursor + 1]]);

        return Ok(length as usize);
    }

    fn check_prelude(&mut self) -> Result<()> {
        // The JPEG File Interchange Format requires the APP0 marker right after the SOI marker.
        let markers = u8x4::from_slice(&self.mmap[self.cursor..self.cursor + 4]);
        self.cursor += 4;

        let expected_markers = u8x4::from_array([0xFF, 0xD8, 0xFF, 0xE0]);
        let mask_markers = markers.simd_eq(expected_markers);

        match mask_markers.all() {
            true => Ok(()),
            false => {
                self.cursor -= 4;
                Err(anyhow!("Expected the SOI marker and APP0 marker."))
            }
        }
    }

    fn check_postlude(&mut self) -> Result<()> {
        let eoi_marker = u8x2::from_slice(&self.mmap[self.mmap.len()-2..]);
        let expected = u8x2::from_array([0xFF, 0xD9]);

        match eoi_marker.simd_eq(expected).all() {
            true => Ok(()),
            false => Err(anyhow!("Expected the EOI marker to appear as the last two bytes in image data"))
        }
    }

    fn parse_headers(&mut self) -> Result<()> {
        if !self.within_bound(2) {
            return Err(anyhow!(
                "we've reached the eof, unable to read header length"
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
        let mut identifier_slice = &self.mmap[self.cursor..self.cursor + 5];
        temp_array[..identifier_slice.len()].copy_from_slice(identifier_slice);

        let identifier = u8x8::from_array(temp_array);
        let expected_identifier = u8x8::from([b'J', b'F', b'I', b'F', 0x00, 0, 0, 0]);

        if !identifier.simd_eq(expected_identifier).all() {
            return Err(anyhow!("identifier was not equal to expected"));
        }

        self.cursor += length;

        Ok(())
    }

    pub fn read(&mut self) -> Result<()> {
        self.check_prelude()?;
        self.parse_headers()?;
        self.check_postlude()?;

        let decoder = JsonDecoder::new(&self.mmap[self.cursor..self.mmap.len() - 2]);

        decoder.decode()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse() -> Result<()> {
        let mut jpeg_reader = JpegReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.read().is_ok());
        Ok(())
    }

    #[test]
    fn test_check_prelude() -> Result<()> {
        let mut jpeg_reader = JpegReader::from_file_path("mike.jpg")?;
        assert!(jpeg_reader.check_prelude().is_ok());
        Ok(())
    }
}
