use crate::huffman_tree::{CodeFreq, HuffmanTree};
use crate::jfif_reader::{MarLen, MARKER_BYTES};
use anyhow::{anyhow, Result};
use std::iter;
use std::simd::prelude::*;
use crate::quant_tables::QuantTable;

const INFORMATION_BYTES: usize = 1;
const HUFFMAN_SYM_BYTES: usize = 16;

pub const QUANTIZATION_TABLE_BYTES: usize = 64;

pub struct JpegDecoder {
    buffer: Vec<u8>,
    huffman_marlen: Vec<MarLen>,
    qt_marlen: Vec<MarLen>,
}

impl JpegDecoder {
    pub fn new(buffer: &[u8], huffman_marlen: Vec<MarLen>, qt_marlen: Vec<MarLen>) -> Self {
        JpegDecoder {
            buffer: buffer.to_vec(),
            huffman_marlen,
            qt_marlen,
        }
    }

    pub fn decode(&self) -> Result<()> {
        let huffman_trees = self.decode_huffman_trees()?;
        let quant_tables = self.decode_quant_table()?;



        Ok(())
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
        let ht_number_mask = Simd::splat(0b1111);
        let ht_numbers = ht_informations & ht_number_mask;

        // extract ht type (bit 4)
        let ht_type_mask = Simd::splat(0b10000);
        let ht_types = (ht_informations & ht_type_mask) >> 4;

        let ht_numbers = ht_numbers.to_array();
        let ht_types = ht_types.to_array();

        Ok((ht_types, ht_numbers))
    }

    fn decode_quant_table_information(&self) -> Result<([u8; 2], [u8; 2])> {
        let qt_informations: Simd<u8, 2> = Simd::from_slice(
            &self
                .qt_marlen
                .iter()
                .map(|marlen| self.buffer[marlen.offset])
                .collect::<Vec<u8>>(),
        );

        // extract ht information
        let qt_precisions_mask = Simd::splat(0b1111);
        let qt_precisions = qt_informations & qt_precisions_mask;

        let qt_ids_mask = Simd::splat(0b11110000);
        let qt_ids = (qt_informations & qt_ids_mask) >> 4;

        let qt_precisions = qt_precisions.to_array();
        let qt_ids = qt_ids.to_array();

        Ok((qt_ids, qt_precisions))
    }

    pub fn decode_quant_table(&self) -> Result<Vec<QuantTable>> {
        debug_assert_eq!(self.qt_marlen.len(), 2);

        let mut tables = vec![];

        let (qt_ids, qt_precisions) = self.decode_quant_table_information()?;

        for (idx, marlen) in self.qt_marlen.iter().enumerate() {
            let MarLen { offset, length } = marlen;

            let mut current_offset = offset + MARKER_BYTES;
            debug_assert!(self.buffer.len() <= current_offset + QUANTIZATION_TABLE_BYTES);

            let qt_data: Simd<u8, QUANTIZATION_TABLE_BYTES> = Simd::from_slice(
                &self.buffer[current_offset..current_offset + QUANTIZATION_TABLE_BYTES],
            );

            let (qt_id, qt_precision) = (qt_ids[idx], qt_precisions[idx]);
            tables.push(QuantTable::from(qt_id, qt_precision, qt_data))
        }

        Ok(tables)
    }

    pub fn decode_huffman_trees(&self) -> Result<Vec<HuffmanTree>> {
        debug_assert_eq!(self.huffman_marlen.len(), 4);

        let mut trees = vec![];

        let (ht_types, ht_numbers) = self.decode_huffman_information()?;

        for (idx, marlen) in self.huffman_marlen.iter().enumerate() {
            let MarLen { offset, length } = marlen;

            let mut current_offset = offset + INFORMATION_BYTES;

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
                .zip(flat_lengths.iter())
                .map(|(&code, &freq)| CodeFreq { code, freq })
                .collect::<Vec<CodeFreq>>();

            let tree = HuffmanTree::from(ht_types[idx], ht_numbers[idx] as usize, code_freq);
            trees.push(tree);
        }

        Ok(trees)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jfif_reader::JFIFReader;
    use memmap::Mmap;
    use std::fs::File;

    #[test]
    fn test_decode_huffman_trees() -> Result<()> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        assert!(jfif_reader.parse().is_ok());

        Ok(())
    }
}
