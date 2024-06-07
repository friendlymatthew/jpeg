#![feature(portable_simd)]

extern crate core;

/// The decoder takes as input compressed image data and table specifications, and by means of a
/// specific set of procedures generates as output `digital reconstructed image data`.
pub mod decoder;


mod coding;
pub(crate) mod frame_header;
pub(crate) mod scan_header;
pub(crate) mod huffman_tree;
pub(crate) mod marker;
pub(crate) mod parser;
pub(crate) mod quantization_table;
pub(crate) mod sample_precision;


/// Features
mod grayscale;
