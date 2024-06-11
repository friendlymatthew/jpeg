use crate::coding::EntropyCoding;
use crate::huffman_tree::{HuffmanClass, };
use crate::scan_header::ScanHeader;
use anyhow::{anyhow, Result};

pub(crate) struct EntropyDecoder<'a> {
    data: &'a [u8],
    cursor: usize,
    scan_header: ScanHeader,
    entropy_coding: EntropyCoding,
}

impl<'a> EntropyDecoder<'a> {
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
                            image_data.push((component_batch[0], component_batch[1], component_batch[2]));
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