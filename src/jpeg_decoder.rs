use crate::jfif_reader::MarLen;
use anyhow::{anyhow, Result};
use std::iter;
use std::simd::prelude::*;

const HUFFMAN_INFORMATION_BYTES: usize = 1;
const HUFFMAN_SYM_BYTES: usize = 16;

pub struct JpegDecoder {
    buffer: Vec<u8>,
    huffman_marlen: Vec<MarLen>,
    dqt_marlen: Vec<MarLen>,
}

impl JpegDecoder {
    pub fn new(buffer: &[u8], huffman_marlen: Vec<MarLen>, dqt_marlen: Vec<MarLen>) -> Self {
        JpegDecoder {
            buffer: buffer.to_vec(),
            huffman_marlen,
            dqt_marlen,
        }
    }

    fn decode_huffman_information(&self) -> Result<([u8; 4], [u8; 4])> {
        let ht_informations: Simd<u8, 4> = Simd::from_slice(
            &self
                .huffman_marlen
                .iter()
                .map(|marlen| self.buffer[marlen.offset])
                .collect::<Vec<u8>>(),
        );

        // extract ht information
        let ht_number_mask = Simd::splat(0x0F);
        let ht_numbers = ht_informations & ht_number_mask;

        // extract ht type (bit 4)
        let ht_type_mask = Simd::splat(0x10);
        let ht_types = (ht_informations & ht_type_mask) >> 4;

        let ht_numbers = ht_numbers.to_array();
        let ht_types = ht_types.to_array();

        Ok((ht_types, ht_numbers))
    }

    pub fn decode_huffman_tables(&self) -> Result<()> {
        let (ht_types, ht_numbers) = self.decode_huffman_information()?;

        for marlen in &self.huffman_marlen {
            let MarLen { offset, length } = marlen;

            let mut current_offset = offset + HUFFMAN_INFORMATION_BYTES;

            if self.buffer.len() < current_offset + HUFFMAN_SYM_BYTES {
                return Err(anyhow!("Not enough data to extract symbol table"));
            }

            let sym_table = &self.buffer[current_offset..current_offset + HUFFMAN_SYM_BYTES];

            let mut flat_lengths = vec![];

            for (idx, mult) in sym_table.iter().enumerate() {
                flat_lengths.extend(iter::repeat(idx + 1).take(*mult as usize));
            }

            current_offset += HUFFMAN_SYM_BYTES;

            let code_len = (offset + length) - current_offset;
            debug_assert_eq!(current_offset + code_len, offset + length);

            let code_freq = self.buffer[current_offset..current_offset + code_len]
                .iter()
                .zip(flat_lengths.iter());

            code_freq.for_each(|(code, freq)| {
                println!("Code: {:?}, frequency: {:?}", code, freq);
            })
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jfif_reader::JFIFReader;
    use memmap::Mmap;
    use std::fs::File;

    #[test]
    fn test_decode_huffman_tables() -> Result<()> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        assert!(jfif_reader.parse().is_ok());

        Ok(())
    }
}
