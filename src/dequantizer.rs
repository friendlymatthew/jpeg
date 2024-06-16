use std::collections::HashMap;
use std::simd::Simd;

use anyhow::{anyhow, Result};

use crate::frame_header::FrameHeader;
use crate::quantization_table::QuantizationTable;

pub(crate) struct Dequantizer<'a> {
    frame_header: &'a FrameHeader,
    data: &'a Vec<([u8; 64], [u8; 64], [u8; 64])>,
    cursor: usize,
    scan_component_order: &'a Vec<u8>,
    quantization_table_map: HashMap<u8, QuantizationTable>,
}

impl<'a> Dequantizer<'a> {
    pub(crate) fn new(
        frame_header: &'a FrameHeader,
        data: &'a Vec<([u8; 64], [u8; 64], [u8; 64])>,
        scan_component_order: &'a Vec<u8>,
        quantization_table_map: HashMap<u8, QuantizationTable>,
    ) -> Self {
        Dequantizer {
            frame_header,
            data,
            cursor: 0,
            scan_component_order,
            quantization_table_map,
        }
    }

    pub(crate) fn dequantize(&mut self) -> Result<Vec<(Simd<u8, 64>, Simd<u8, 64>, Simd<u8, 64>)>> {
        let mut dequantized_coefficients = vec![];

        for mcu in self.data {
            let (c1, c2, c3) = *mcu;
            let idct: Result<Vec<_>> = self
                .scan_component_order
                .iter()
                .zip(vec![c1, c2, c3].iter())
                .map(|(component_id, mcu)| {
                    let QuantizationTable {
                        quantization_table_element,
                        ..
                    } = *self
                        .quantization_table_map
                        .get(component_id)
                        .ok_or(anyhow!(format!(
                            "failed to find component id {}",
                            component_id
                        )))?;

                    Ok(Simd::from_array(*mcu) * quantization_table_element)
                })
                .collect();

            let idct = idct?;
            debug_assert_eq!(idct.len(), 3);
            dequantized_coefficients.push((idct[0], idct[1], idct[2]))
        }

        Ok(dequantized_coefficients)
    }
}
