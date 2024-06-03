#![feature(portable_simd)]

extern crate core;

pub mod decoder;
mod grayscale;
pub(crate) mod huffman_tree;
pub(crate) mod marker;
pub(crate) mod parser;
pub(crate) mod quantization_table;
pub(crate) mod sample_precision;

pub enum EntropyCoding {
    Huffman,
    Arithmetic,
}

#[derive(Debug, Copy, Clone)]
pub enum EncodingProcess {
    BaselineDCT,
    ExtendedSequentialDCT,
    ProgressiveDCT,
    LosslessSequential,
}

use crate::sample_precision::SamplePrecision;

#[derive(Debug)]
pub struct FrameHeader {
    /// Specifies the precision in bits for the samples of the components in the frame
    pub(crate) precision: SamplePrecision,

    /// Number of lines -- Specifies the maximum number of lines in the source image. This shall
    /// be equal to the number of lines in the component with the maximum number of vertical samples.
    pub(crate) image_height: usize,

    /// Number of samples per line -- Specifies the maximum number of samples per line in the source
    /// image. This shall be equal to the number of lines the component with the maximum number
    /// of vertical samples.
    pub(crate) image_width: usize,

    ///
    pub(crate) component_type: ComponentType,
    pub(crate) components: Vec<Component>,
}

#[derive(Debug)]
pub struct ScanData {
    pub(crate) component_id: u8,
    pub(crate) dc_table_id: u8,
    pub(crate) ac_table_id: u8,
}

impl ScanData {
    pub(crate) fn from(component_id: u8, dc_table_id: u8, ac_table_id: u8) -> Self {
        ScanData {
            component_id,
            dc_table_id,
            ac_table_id,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ComponentType {
    Grayscale,
    Color,
}

impl ComponentType {
    pub(crate) fn from(b: u8) -> Self {
        match b {
            1 => ComponentType::Grayscale,
            3 => ComponentType::Color,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Component {
    pub(crate) component_id: u8,
    pub(crate) horizontal_scaling_factor: u8,
    pub(crate) vertical_scaling_factor: u8,
    pub(crate) qt_table_id: u8,
}

impl Component {
    pub(crate) fn from(
        component_id: u8,
        horizontal_sf: u8,
        vertical_sf: u8,
        qt_table_id: u8,
    ) -> Self {
        Component {
            component_id,
            horizontal_scaling_factor: horizontal_sf,
            vertical_scaling_factor: vertical_sf,
            qt_table_id,
        }
    }
}
