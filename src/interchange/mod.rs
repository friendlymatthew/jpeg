use std::collections::HashMap;
use std::hash::Hash;
use crate::entropy::EntropyCoding;
use crate::interchange::marker::Marker;
use sample_precision::SamplePrecision;
use anyhow::Result;
pub(crate) mod marker;
pub(crate) mod reader;
pub(crate) mod sample_precision;


pub(crate) enum Scan {
    Interleaved,
    NonInterleaved
}

pub(crate) enum Operation {
    SequentialDCT,
    ProgressiveDCT,
    // Lossless,
    // Hierarchical
}

pub(crate) enum Compression {
    Baseline,
    ExtendedDCT,
    // Lossless,
    // Hierarchical Process
}

pub enum ProcessType {
    DCT,
    // Predictive Process (lossless)
    // Multiple Frames (non-differential & differential)
}

pub struct CompressionSpecification {
    process_type: ProcessType,
    precision: Vec<SamplePrecision>,
    operations: Vec<Operation>,
    entropy_codings: Vec<EntropyCoding>,
    scans: Vec<Scan>,
    num_ac_tables: usize,
    num_dc_tables: usize,
    num_components: usize,
}

impl Compression {
    fn validate_marlen_map(&self, marlen_map: &HashMap<Marker, Vec<(usize, usize)>>) -> Result<()> {


        Ok(())
    }

    fn specification(&self) -> CompressionSpecification {
        match self {
            Compression::Baseline => CompressionSpecification {
                process_type: ProcessType::DCT,
                precision: vec![SamplePrecision::EightBit],
                operations: vec![Operation::SequentialDCT],
                entropy_codings: vec![EntropyCoding::Huffman],
                num_ac_tables: 2,
                num_dc_tables: 2,
                scans: vec![Scan::Interleaved, Scan::NonInterleaved],
                num_components: 4,
            },
            Compression::ExtendedDCT => CompressionSpecification {
                process_type: ProcessType::DCT,
                precision: vec![SamplePrecision::EightBit, SamplePrecision::SixteenBit],
                operations: vec![Operation::SequentialDCT, Operation::ProgressiveDCT],
                entropy_codings: vec![EntropyCoding::Huffman, EntropyCoding::Arithmetic],
                num_ac_tables: 4,
                num_dc_tables: 4,
                scans: vec![Scan::NonInterleaved, Scan::Interleaved],
                num_components: 4
            }
        }
    }
}
