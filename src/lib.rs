#![feature(portable_simd)]

extern crate core;

mod coding;
pub mod decoder;
mod frame_header;
mod grayscale;
pub(crate) mod huffman_tree;
pub(crate) mod marker;
pub(crate) mod parser;
pub(crate) mod quantization_table;
pub(crate) mod sample_precision;
mod scan_header;
