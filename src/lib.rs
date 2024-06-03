#![feature(portable_simd)]

extern crate core;

pub mod decoder;
mod frame_header;
mod grayscale;
pub(crate) mod huffman_tree;
pub(crate) mod marker;
pub(crate) mod parser;
pub(crate) mod quantization_table;
pub(crate) mod sample_precision;
mod scan_header;

pub enum EntropyCoding {
    Huffman,
    Arithmetic,
}

#[derive(Debug, Copy, Clone)]
pub enum EncodingProcess {
    BaselineDCT,
    ExtendedSequentialDCT,
    ProgressiveDCT,
    LosslessSequential,
}
