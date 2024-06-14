use anyhow::{anyhow, Result};

use crate::coding::EntropyCoding;
use crate::huffman_tree::HuffmanClass;
use crate::scan_header::ScanHeader;

pub(crate) struct EntropyDecoder<'a> {
    data: &'a [u8],
    cursor: usize,
    scan_header: ScanHeader,
    entropy_coding: EntropyCoding,
}

impl<'a> EntropyDecoder<'a> {
    const ZIGZAG_TABLE: [usize; 64] = [
        0, 1, 5, 6, 14, 15, 27, 28, 2, 4, 7, 13, 16, 26, 29, 42, 3, 8, 12, 17, 25, 30, 41, 43, 9,
        11, 18, 24, 31, 40, 44, 53, 10, 19, 23, 32, 39, 45, 52, 54, 20, 22, 33, 38, 46, 51, 55, 60,
        21, 34, 37, 47, 50, 56, 59, 61, 35, 36, 48, 49, 57, 58, 62, 63,
    ];

    pub(crate) fn new(
        data: &'a [u8],
        scan_header: ScanHeader,
        entropy_coding: EntropyCoding,
    ) -> Self {
        EntropyDecoder {
            data,
            cursor: 0,
            scan_header,
            entropy_coding,
        }
    }

    pub(crate) fn zigzag(&mut self, data: Vec<(u8, u8, u8)>) -> Result<Vec<[(u8, u8, u8); 64]>> {
        self.cursor = 0;
        let mut unzigzagged = vec![];

        let mut temp_chunk = [(0u8, 0u8, 0u8); 64];

        while self.cursor < data.len() {
            let end = (self.cursor + 64).min(data.len());
            let len = end - self.cursor;

            temp_chunk[..len].copy_from_slice(&data[self.cursor..end]);

            let mut new_chunk = [(0u8, 0u8, 0u8); 64];
            temp_chunk.into_iter().enumerate().for_each(|(idx, block)| {
                let jdx = Self::ZIGZAG_TABLE[idx];
                new_chunk[jdx] = block;
            });

            unzigzagged.push(new_chunk);

            self.cursor += len;
        }

        Ok(unzigzagged)
    }

    pub(crate) fn decode(&mut self) -> Result<Vec<(u8, u8, u8)>> {
        let uncompressed_image_data = match &self.entropy_coding {
            EntropyCoding::Huffman(_) => self.decode_huffman(),
            EntropyCoding::Arithmetic(_) => todo!(),
        }?;

        Ok(uncompressed_image_data)
    }

    fn decode_huffman(&mut self) -> Result<Vec<(u8, u8, u8)>> {
        let mut image_data = vec![];
        let huffman_map = self.entropy_coding.huffman_map();

        let ac_dc_destination_ids: Vec<_> = self
            .scan_header
            .scan_component_selectors
            .iter()
            .map(|s| (s.dc_destination_id, s.ac_destination_id))
            .collect();

        let mut component_ptr = 0;
        let mut num_coeffs = 0;

        let mut node_cursor = *huffman_map
            .get(&(HuffmanClass::DC, ac_dc_destination_ids[component_ptr].0))
            .ok_or(anyhow!(format!(
                "failed to find a component with id: {component_ptr}"
            )))?;

        let mut component_batch = vec![];
        while self.cursor < self.data.len() {
            if let Some(node) = node_cursor {
                unsafe {
                    if (*node.as_ptr()).code != u8::MAX {
                        component_batch.push((*node.as_ptr()).code);
                        component_ptr += 1;

                        if component_ptr == ac_dc_destination_ids.len() {
                            component_ptr = 0;
                            num_coeffs += 1;

                            debug_assert_eq!(component_batch.len(), 3);
                            image_data.push((
                                component_batch[0],
                                component_batch[1],
                                component_batch[2],
                            ));
                            component_batch.clear();
                        }

                        let (next_class, next_destination_id) = if num_coeffs % 64 == 0 {
                            (HuffmanClass::DC, ac_dc_destination_ids[component_ptr].0)
                        } else {
                            (HuffmanClass::AC, ac_dc_destination_ids[component_ptr].1)
                        };

                        node_cursor =
                            *huffman_map
                                .get(&(next_class, next_destination_id))
                                .ok_or(anyhow!(format!(
                                    "failed to find a component with id: {component_ptr}"
                                )))?;
                    } else {
                        match self.data[self.cursor] {
                            0 => {
                                node_cursor = (*node.as_ptr()).left;
                            }
                            1 => {
                                node_cursor = (*node.as_ptr()).right;
                            }
                            _ => unreachable!(),
                        };
                    }
                }
            }

            self.cursor += 1;
        }

        println!("image data: {:?}", image_data.len());
        Ok(image_data)
    }
}

#[cfg(test)]
mod tests {
    use crate::frame_header::ComponentType;
    use crate::scan_header::EncodingOrder;

    use super::*;

    #[test]
    fn test_zigzag() -> Result<()> {
        let mut entropy_decoder = EntropyDecoder {
            data: &[],
            cursor: 0,
            scan_header: ScanHeader {
                encoding_order: EncodingOrder::NonInterleaved,
                component_type: ComponentType::Grayscale,
                scan_component_selectors: vec![],
                start_of_spectral: 0,
                end_of_spectral: 0,
                successive_approx_bit_position_high: 0,
                point_transform: 0,
            },
            entropy_coding: EntropyCoding::Huffman(vec![]),
        };

        let data = vec![
            (0, 0, 0),
            (1, 1, 1),
            (2, 2, 2),
            (3, 3, 3),
            (4, 4, 4),
            (5, 5, 5),
            (6, 6, 6),
            (7, 7, 7),
            (8, 8, 8),
            (9, 9, 9),
            (10, 10, 10),
            (11, 11, 11),
            (12, 12, 12),
            (13, 13, 13),
            (14, 14, 14),
            (15, 15, 15),
            (16, 16, 16),
            (17, 17, 17),
            (18, 18, 18),
            (19, 19, 19),
            (20, 20, 20),
            (21, 21, 21),
            (22, 22, 22),
            (23, 23, 23),
            (24, 24, 24),
            (25, 25, 25),
            (26, 26, 26),
            (27, 27, 27),
            (28, 28, 28),
            (29, 29, 29),
            (30, 30, 30),
            (31, 31, 31),
            (32, 32, 32),
            (33, 33, 33),
            (34, 34, 34),
            (35, 35, 35),
            (36, 36, 36),
            (37, 37, 37),
            (38, 38, 38),
            (39, 39, 39),
            (40, 40, 40),
            (41, 41, 41),
            (42, 42, 42),
            (43, 43, 43),
            (44, 44, 44),
            (45, 45, 45),
            (46, 46, 46),
            (47, 47, 47),
            (48, 48, 48),
            (49, 49, 49),
            (50, 50, 50),
            (51, 51, 51),
            (52, 52, 52),
            (53, 53, 53),
            (54, 54, 54),
            (55, 55, 55),
            (56, 56, 56),
            (57, 57, 57),
            (58, 58, 58),
            (59, 59, 59),
            (60, 60, 60),
            (61, 61, 61),
            (62, 62, 62),
            (63, 63, 63),
        ];

        let unzigzagged = entropy_decoder.zigzag(data)?;

        println!("{:?}", unzigzagged);

        Ok(())
    }
}
