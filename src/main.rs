#![feature(portable_simd)]

use image::{GenericImageView, ImageBuffer, Rgba};
use std::simd::prelude::*;

fn realign_rgb_data(src: &[u8], num: usize) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let mut r: Vec<u8> = vec![0; num];
    let mut g: Vec<u8> = vec![0; num];
    let mut b: Vec<u8> = vec![0; num];

    for i in 0..num {
        r[i] = src[i * 4];
        g[i] = src[i * 4 + 1];
        b[i] = src[i * 4 + 2];
    }

    (r, g, b)
}

fn rgb_to_grayscale(src: &[u8], dst: &mut [u8], num: usize) {
    let simd_size = 64;

    let (r, g, b) = realign_rgb_data(src, num);

    let r_factor = f32x64::splat(0.29891);
    let g_factor = f32x64::splat(0.58661);
    let b_factor = f32x64::splat(0.11448);

    for i in (0..num).step_by(simd_size) {
        let end = (i + simd_size).min(num);
        let len = end - i;
        let mut r_chunk = [0u8; 64];
        let mut g_chunk = [0u8; 64];
        let mut b_chunk = [0u8; 64];

        r_chunk[..len].copy_from_slice(&r[i..end]);
        g_chunk[..len].copy_from_slice(&g[i..end]);
        b_chunk[..len].copy_from_slice(&b[i..end]);

        let r_chunk = u8x64::from_array(r_chunk);
        let g_chunk = u8x64::from_array(g_chunk);
        let b_chunk = u8x64::from_array(b_chunk);

        let r_f32 = r_chunk.cast();
        let g_f32 = g_chunk.cast();
        let b_f32 = b_chunk.cast();

        let gray = r_f32 * r_factor + g_f32 * g_factor + b_f32 * b_factor;
        let gray_u8 = gray.cast::<u8>();

        for j in 0..len {
            let idx = i + j;
            let y = gray_u8[j];
            dst[idx * 4] = y;
            dst[idx * 4 + 1] = y;
            dst[idx * 4 + 2] = y;
            dst[idx * 4 + 3] = 255;
        }
    }
}

fn main() {
    let img = image::open("mike.jpg").unwrap().to_rgba8();
    let (w, h) = img.dimensions();
    let num_pxs = (w * h) as usize;

    let mut gray_img_buffer = vec![0; num_pxs * 4];
    rgb_to_grayscale(&img, &mut gray_img_buffer, num_pxs);

    let buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(w, h, gray_img_buffer).unwrap();
    buffer.save("gray_mike.png").unwrap();
}
