use crate::scan_header::EncodingOrder;
use anyhow::Result;

pub struct BitReader<'a> {
    encoding_order: EncodingOrder,
    pub(crate) data: &'a [u8],
    pub(crate) bit_cur: usize,
    pub(crate) byte_cur: usize,
}

impl<'a> BitReader<'a> {
    pub(crate) fn new(data: &'a [u8], encoding_order: EncodingOrder) -> Self {
        BitReader {
            encoding_order,
            data,
            bit_cur: 0,
            byte_cur: 0,
        }
    }

    fn u8_to_bits(&mut self, byte: u8) -> [u8; 8] {
        let mut bits = [0; 8];
        for i in 0..8 {
            bits[i] = (byte >> (7 - i)) & 1;
        }
        bits
    }

    fn slice_to_bits(&mut self) -> Vec<u8> {
        let mut bits = Vec::with_capacity(self.data.len() * 8);
        for &byte in self.data {
            bits.extend_from_slice(&self.u8_to_bits(byte));
        }
        bits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_u8_to_bits() -> Result<()> {
        let test_cases = vec![
            (4, vec![0, 0, 0, 0, 0, 1, 0, 0]),
            (21, vec![0, 0, 0, 1, 0, 1, 0, 1]),
            (69, vec![0, 1, 0, 0, 0, 1, 0, 1]),
        ];

        for (num, expected) in test_cases {
            let data = vec![num];
            let mut bit_reader = BitReader::new(&data, EncodingOrder::Interleaved);

            let got = bit_reader.u8_to_bits(num);

            assert_eq!(got.to_vec(), expected)
        }

        Ok(())
    }

    #[test]
    fn test_slice_to_bits() -> Result<()> {
        let data = vec![4, 21, 69];

        let mut bit_reader = BitReader::new(&data, EncodingOrder::Interleaved);
        let got = bit_reader.slice_to_bits();

        assert_eq!(
            got,
            vec![0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, 0, 1]
        );

        Ok(())
    }
}
