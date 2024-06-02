use std::simd::Simd;
use crate::decoder::component::{FrameData, ScanData};
use crate::entropy::huffman_table::HuffmanTree;
use crate::quantize::quantization_table::QuantTable;

pub struct Block(Simd<u8, 64>);

impl Block {
    pub(crate) const WIDTH: usize = 8;
}

pub(crate) struct Image {
    pub(crate) data: Vec<u8>,
    pub(crate) huffman_trees: Vec<HuffmanTree>,
    pub(crate) quant_tables: Vec<QuantTable>,
    pub(crate) start_of_frame: FrameData,
    pub(crate) start_of_scan: Vec<ScanData>,
}

impl Image {
    pub(crate) fn build(&self) {
        let FrameData { .. } = self.start_of_frame;

        // debug_assert_eq!(image_height * image_width, self.data.len());

        const LANE_COUNT: usize = 64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use memmap::Mmap;
    use std::fs::File;
    use crate::decoder::baseline_process::decoder::JpegDecoder;
    use crate::interchange::Compression;
    use crate::interchange::reader::JFIFReader;

    fn mike_decoder() -> anyhow::Result<JpegDecoder> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        Ok(jfif_reader.decoder(Compression::Baseline)?)
    }

    #[test]
    fn test_image_data() -> anyhow::Result<()> {
        let decoder = mike_decoder()?;
        let image = decoder.decode()?;

        image.build();

        Ok(())
    }
}
