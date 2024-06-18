use std::f32::consts::PI;

use crate::sample_precision::SamplePrecision;

pub(crate) struct IDCT {
    pub(crate) table: [f32; 64],
    pub(crate) precision: SamplePrecision,
}

impl IDCT {
    fn norm_coeff(u: usize) -> f32 {
        match u {
            0 => ((1.0 / 2.0) as f32).sqrt(),
            _ => 1.0,
        }
    }

    pub(crate) fn new(sample_precision: SamplePrecision) -> Self {
        let mut idct_table = [0.0; 64];

        for u in 0..8 {
            for x in 0..8 {
                idct_table[u * 8 + x] =
                    Self::norm_coeff(u) * ((2.0 * x as f32 + 1.0) * u as f32 * PI / 16.0).cos()
            }
        }

        Self {
            table: idct_table,
            precision: sample_precision,
        }
    }

    /// todo refactor this!
    pub(crate) fn perform_idct(&self, mcu: [f32; 64]) -> [f32; 64] {
        let mut output = [0f32; 64];

        for x in 0..8 {
            for y in 0..8 {
                let mut local_sum = 0.0;

                for u in 0..self.precision as usize {
                    for v in 0..self.precision as usize {
                        let cu = if u == 0 {
                            ((1.0 / 2.0) as f32).sqrt()
                        } else {
                            1.0
                        };
                        let cv = if v == 0 {
                            ((1.0 / 2.0) as f32).sqrt()
                        } else {
                            1.0
                        };
                        let dct_coeff = mcu[u * 8 + v];
                        local_sum +=
                            cu * cv * dct_coeff * self.table[u * 8 + x] * self.table[v * 8 + y];
                    }
                }

                output[x * 8 + y] = 0.25 * local_sum;
            }
        }

        output
    }
}
