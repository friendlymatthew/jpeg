#![feature(portable_simd)]

extern crate core;

mod component;
mod jpeg_decoder;
mod quant_tables;
mod decoder;
mod reader;
mod marker;
mod features;
mod huffman_table;