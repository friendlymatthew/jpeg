use std::simd::Simd;
use crate::component::{FrameData, ScanData};
use crate::huffman_tree::HuffmanTree;
use crate::quant_tables::QuantTable;

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
        let FrameData { image_width, image_height, .. } = self.start_of_frame;


        todo!()
    }

}