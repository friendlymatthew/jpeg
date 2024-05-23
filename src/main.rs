#![feature(portable_simd)]

mod rgb_to_grayscale;

use image::{GenericImageView, ImageBuffer, Rgba};
use crate::rgb_to_grayscale::rgb_to_grayscale;


fn main() {
    let img = image::open("mike.jpg").unwrap().to_rgba8();
    let (w, h) = img.dimensions();
    let num_pxs = (w * h) as usize;

    let mut gray_img_buffer = vec![0; num_pxs * 4];
    rgb_to_grayscale(&img, &mut gray_img_buffer, num_pxs);

    let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(w, h, gray_img_buffer).unwrap();
    buffer.save("gray_mike.png").unwrap();
}
