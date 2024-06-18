use std::simd::Simd;

use crate::color_spaces::ColorSpace::RGB;

type MCU = (Simd<f32, 64>, Simd<f32, 64>, Simd<f32, 64>);

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ColorSpace {
    YCbCr(f32, f32, f32),
    RGB(f32, f32, f32),
}

impl ColorSpace {
    pub(crate) fn convert_ycbcr_to_rgb(image_data: Vec<MCU>) -> Vec<Self> {
        let mut rgbs = vec![];

        image_data.iter().for_each(|(ys, cbs, crs)| {
            let cbs = cbs - Simd::splat(128.0);
            let crs = crs - Simd::splat(128.0);

            let rs = ys + Simd::splat(1.402) * crs;
            let gs = ys - Simd::splat(0.344136) * cbs - Simd::splat(0.714136) * crs;
            let bs = ys + Simd::splat(1.772) * cbs;

            rgbs = rs
                .to_array()
                .iter()
                .zip(gs.to_array().iter())
                .zip(bs.to_array().iter())
                .map(|((r, g), b)| RGB(*r, *g, *b))
                .collect();
        });

        rgbs
    }
}
