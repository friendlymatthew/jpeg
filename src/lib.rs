#![feature(portable_simd)]

extern crate core;

mod component;
mod decoder;
mod entropy;
mod features;
mod huffman_table;
mod marker;
mod quant_tables;
mod reader;
