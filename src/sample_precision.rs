#[derive(Debug, PartialEq, Copy, Clone)]
pub(crate) enum SamplePrecision {
    EightBit,
    SixteenBit,
}

impl SamplePrecision {
    pub(crate) fn decode(b: u8) -> Self {
        match b {
            0 => SamplePrecision::EightBit,
            1 => SamplePrecision::SixteenBit,
            _ => unreachable!(),
        }
    }

    pub(crate) fn parse(number_of_bits: u8) -> Self {
        match number_of_bits {
            8 => SamplePrecision::EightBit,
            16 => SamplePrecision::SixteenBit,
            _ => unreachable!(),
        }
    }
}
