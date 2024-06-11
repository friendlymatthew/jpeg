use crate::coding::EntropyCoding;
use crate::frame_header::FrameHeader;
use crate::huffman_tree::HuffmanNode;
use crate::scan_header::ScanHeader;
use anyhow::{anyhow, Result};

pub(crate) struct EntropyDecoder<'a> {
    data: &'a [u8],
    cursor: usize,
    scan_header: ScanHeader,
    frame_header: FrameHeader,
    entropy_coding: EntropyCoding,
}

impl<'a> EntropyDecoder<'a> {
    pub(crate) fn new(
        data: &'a [u8],
        scan_header: ScanHeader,
        frame_header: FrameHeader,
        entropy_coding: EntropyCoding,
    ) -> Self {
        EntropyDecoder {
            data,
            cursor: 0,
            scan_header,
            frame_header,
            entropy_coding,
        }
    }

    pub(crate) fn decode(&mut self) -> Result<Vec<u8>> {
        let uncompressed_image_data = match &self.entropy_coding {
            EntropyCoding::Huffman(_) => self.decode_huffman(),
            EntropyCoding::Arithmetic(_) => todo!(),
        }?;

        Ok(uncompressed_image_data)
    }

    fn decode_huffman(&mut self) -> Result<Vec<u8>> {
        let mut image_data = vec![];

        let huffman_map = self.entropy_coding.huffman_map();
        println!("huffman map keys: {:?}", huffman_map.keys());

        let component_ord_ids: Vec<_> = self
            .scan_header
            .scan_component_selectors
            .iter()
            .map(|s| s.component_id)
            .collect();

        println!("number of components {:?}", component_ord_ids);

        let mut component_ptr = 0;

        let mut current_root = *huffman_map
            .get(&component_ord_ids[component_ptr])
            .ok_or(anyhow!(format!("failed to find a component with id: {component_ptr}")))?;
        while self.cursor < self.data.len() {
            match self.data[self.cursor] {
                0 => {
                    if let Some(ptr) = current_root {
                        current_root = unsafe { (*ptr.as_ptr()).left };
                    }
                }
                1 => {
                    if let Some(node) = current_root {
                        current_root = unsafe { (*node.as_ptr()).right };
                    }
                }
                _ => return Err(anyhow!("data input should only be 1's and 0's")),
            }

            if HuffmanNode::is_leaf(current_root) {
                let decompressed_value = if let Some(node) = current_root {
                    unsafe { (*node.as_ptr()).code }
                } else {
                    return Err(anyhow!("unexpected None pointer after checking valid leaf"));
                };

                image_data.push(decompressed_value);
                component_ptr += 1;

                if component_ptr >= component_ord_ids.len() {
                    component_ptr = 0;
                }
                current_root = *huffman_map
                    .get(&component_ord_ids[component_ptr])
                    .ok_or(anyhow!(format!("failed to find a component with id {}", component_ptr)))?;
            }

            self.cursor += 1;
        }

        Ok(image_data)
    }
}
