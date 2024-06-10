use crate::coding::EntropyCoding;
use crate::frame_header::FrameHeader;
use crate::scan_header::{ScanHeader};
use anyhow::{Result};

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
        let huffman_map = self.entropy_coding.huffman_map();
        let uncompressed_image_data = vec![];

        let component_ord: Vec<_> = self
            .scan_header
            .scan_component_selectors
            .iter()
            .map(|s|
                s.component_id
            ).collect();

        let num_components = component_ord.len();
        let mut component_ptr = 0;




        Ok(uncompressed_image_data)
    }
}


