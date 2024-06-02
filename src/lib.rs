#![feature(portable_simd)]

extern crate core;

mod decoder;
mod entropy;
mod features;
mod quantize;
mod interchange;


pub(crate) struct MinimumCodedUnit {

}