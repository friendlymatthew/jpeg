use crate::huffman_tree::{HuffmanClass, HuffmanTree, NPtr};
use std::collections::HashMap;

pub(crate) enum Operation {
    Sequential,
    Progressive,
}

pub(crate) enum EntropyCoding {
    Huffman(Vec<HuffmanTree>),
    Arithmetic(Vec<()>),
}

/// (table_class, destination_id)
type HuffmanMapKey = (HuffmanClass, u8);

impl EntropyCoding {
    pub(crate) fn huffman_map(&self) -> HashMap<HuffmanMapKey, NPtr> {
        let mut map = HashMap::new();
        match self {
            EntropyCoding::Huffman(hts) => hts.iter().for_each(|ht| {
                map.insert((ht.class, ht.destination_id), ht.root);
            }),
            _ => panic!(),
        };

        map
    }
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
