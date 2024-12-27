use std::simd::Simd;

use crate::sample_precision::SamplePrecision;

/// The set of 64 quantization values used to quantize the DCT coefficients
#[derive(Debug, Copy, Clone)]
pub struct QuantizationTable {
    /// Specifies the precision of the qk values. Value 0 indicates 8-bit Qk values; value 1
    /// indicates 16-bit Qk values. Pq shall be zero for 8 bit sample precision P.
    pub(crate) precision: SamplePrecision,

    /// Specifies one of four possible destinations at the decoder into which the quantization
    /// shall be used.
    pub(crate) table_id: u8,

    /// Specifies the kth element out of 64 elements, where k is the index in the zig-zag ordering
    /// of the DCT coefficients. The quantization elements shall be specified in zig-zag scan order.
    pub(crate) quantization_table_element: Simd<u8, 64>,
}

impl QuantizationTable {
    pub(crate) fn from(qt_id: u8, qt_precision: u8, qt_data: Simd<u8, 64>) -> Self {
        QuantizationTable {
            table_id: qt_id,
            precision: SamplePrecision::decode(qt_precision),
            quantization_table_element: qt_data,
        }
    }
}
