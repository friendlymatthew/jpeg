use crate::frame_header::ComponentType;

/// (Pg. 25) For a given scan, if the scan header parameter `component_type` is 1, then data from only
/// one source component - the component specified by parameter `Components`[0] - shall be present
/// within the scan. If `component_type` > 1, then data from `Components` shall be present within
/// the scan. The order of components in a scan shall be according to the order specified in the
/// `FrameHeader`.
#[derive(Debug)]
pub(crate) enum EncodingOrder {
    /// The encoder compressed all image data units in component A before beginning component B
    NonInterleaved,

    /// The encoder compresses a data unit from A, a data unit from B, a data unit from C, then
    /// back to A, etc...
    Interleaved,
}

#[derive(Debug)]
pub struct ScanHeader {
    pub(crate) encoding_order: EncodingOrder,

    /// NS: Number of image components in frame -- Specifies the number of source image components in
    /// the frame. The value of `num_components` shall be equal to the number of sets of frame
    ///component specification parameters (Ci, Hi, Vi, Tqi) present in the frame header.
    pub(crate) component_type: ComponentType,

    /// Csj: where J is the length of the `Vec<ScanComponentSelector>`.
    /// Selects which of the component type specified in the frame parameters shall be the jth
    /// component in the scan. Each scan component selector shall match one of the Ci values
    /// specified in the frame header.
    pub(crate) scan_component_selectors: Vec<ScanComponentSelector>,

    /// Ss: Start of spectral or predictior selection. In DCT modes of operation, this parameter
    /// specifies the first DCT coefficient in each block in zig-zag order which shall be coded in
    /// the scan. This parameter is set to zero for sequential DCT processes. In the lossless mode
    /// of operations this parameter is used to select the predictor
    pub(crate) start_of_spectral: u8,

    /// Se: Specifies the last DCT coefficient in each block in zig-zag order which shall be coded in
    /// the scan. This parameter shall be set to 63 for sequential DCT processes. In the lossless
    /// mode of operations this parameter has no meaning. It shall be set to zero.
    pub(crate) end_of_spectral: u8,

    /// Ah: This parameter specifies the point transform used in the preceding scan (the successive
    /// approximation bit position low in the preceding scan) for the band of coefficients specified
    /// by `predictor_selection` and `end_of_spectral_selection`. This parameter shall be set to
    /// zero for the first scan of each band of coefficients. In the lossless mode of operations
    /// this parameter has no meaning. It shall be set to zero.
    pub(crate) successive_approx_bit_position_high: u8,

    /// Al:In DCT modes of operation, this parameter specifies the point transform (bit position low,
    /// used before coding the band of coefficients specified by `predictor_selection` and
    /// `end_of_spectral_selection`. This parameter shall be set to zero for sequential DCT processes.
    /// In the lossless mode of operations, this parameter specifies the point transform Pt.
    pub(crate) point_transform: u8,
}

#[derive(Debug)]
pub struct ScanComponentSelector {
    component_id: u8,

    /// Tdj: Specifies one of four possible DC entropy coding table destinations from which the entropy
    /// table needed for decoding of the DC coefficients of component selector j is retrieved.
    dc_destination_id: u8,

    /// Taj: Specifies one of four possible AC entropy coding table destinations from which the entropy
    /// table needed for decoding of the AC coefficients of component selector j is retrieved.
    ac_destination_id: u8,
}

impl ScanComponentSelector {
    pub(crate) fn from(component_id: u8, dc_destination_id: u8, ac_destination_id: u8) -> Self {
        Self {
            component_id,
            dc_destination_id,
            ac_destination_id,
        }
    }
}
