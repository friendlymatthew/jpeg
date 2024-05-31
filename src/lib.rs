#![feature(portable_simd)]

extern crate core;

mod component;
mod decoder;
mod features;
mod huffman_table;
mod jpeg_decoder;
mod marker;
mod quant_tables;
mod reader;
