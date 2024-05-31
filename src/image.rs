use crate::component::{FrameData, ScanData};
use crate::huffman_tree::HuffmanTree;
use crate::quant_tables::QuantTable;
use std::simd::Simd;

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
        let FrameData {
            ..
        } = self.start_of_frame;

        // debug_assert_eq!(image_height * image_width, self.data.len());

        const LANE_COUNT: usize = 64;
    }
}

#[cfg(test)]
mod tests {
    use crate::jfif_reader::JFIFReader;
    use crate::jpeg_decoder::JpegDecoder;
    use memmap::Mmap;
    use std::fs::File;

    fn mike_decoder() -> anyhow::Result<JpegDecoder> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        Ok(jfif_reader.decoder()?)
    }

    #[test]
    fn test_image_data() -> anyhow::Result<()> {
        let decoder = mike_decoder()?;
        let image = decoder.decode()?;

        image.build();

        Ok(())
    }
}
