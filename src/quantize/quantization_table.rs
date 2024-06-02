use std::simd::Simd;
use crate::interchange::sample_precision::SamplePrecision;

#[derive(Debug)]
enum TableType {
    Luminance = 0,
    Chrominance = 1,
}

// 8x8
pub const QUANT_TABLE_WIDTH: usize = 8;

#[derive(Debug)]
pub struct QuantTable {
    table_type: TableType,
    precision: SamplePrecision,
    data: Simd<u8, 64>, // 8x8
}

impl QuantTable {
    pub(crate) fn from(qt_id: u8, qt_precision: u8, qt_data: Simd<u8, 64>) -> Self {
        QuantTable {
            table_type: match qt_id {
                0 => TableType::Luminance,
                1 => TableType::Chrominance,
                _ => unreachable!(),
            },
            precision: SamplePrecision::decode(qt_precision),
            data: qt_data,
        }
    }
}
