use crate::interchange::component::{FrameData, ScanData};
use crate::entropy::EntropyCoding;
use crate::quantize::quantization_table::QuantTable;

pub struct JFIF {
    pub(crate) data: Vec<u8>,
    pub(crate) entropy_coding: EntropyCoding,
    pub(crate) quant_tables: Vec<QuantTable>,
    pub(crate) frame_header: FrameData,
    pub(crate) scan_header: Vec<ScanData>,
}
