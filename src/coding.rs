pub(crate) enum Operation {
    Sequential,
    Progressive,
}

pub(crate) enum EntropyCoding {
    Huffman,
    Arithmetic,
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum CodingProcess {
    BaselineDCT,
    ExtendedSequentialDCT,
}

#[derive(PartialEq)]
pub struct ProcessSchema {
    /// [EightBitPrecision, SixteenBitPrecision]
    pub(crate) precisions: [bool; 2],

    /// [Sequential, Progressive]
    pub(crate) operations: [bool; 2],

    /// [Huffman, Arithmetic]
    pub(crate) entropy_coding: [bool; 2],

    /// (# AC tables, # DC tables)
    pub(crate) entropy_table_count: (usize, usize),
}


impl CodingProcess {
    pub(crate) fn schema(&self) -> ProcessSchema {
        match self {
            CodingProcess::BaselineDCT => ProcessSchema {
                precisions: [true, false],
                operations: [true, false],
                entropy_coding: [true, false],
                entropy_table_count: (2, 2),
            },
            CodingProcess::ExtendedSequentialDCT => ProcessSchema {
                precisions: [true, true],
                operations: [true, true],
                entropy_coding: [true, true],
                entropy_table_count: (4, 4),
            },
        }
    }
}
