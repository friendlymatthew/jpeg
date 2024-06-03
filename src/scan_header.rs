use crate::frame_header::ComponentType;

#[derive(Debug)]
pub struct ScanHeader {
    /// Number of image components in frame -- Specifies the number of source image components in
    /// the frame. The value of `num_components` shall be equal to the number of sets of frame
    ///component specification parameters (Ci, Hi, Vi, Tqi) present in the frame header.
    pub(crate) component_type: ComponentType,

    pub(crate) scan_component_selectors: Vec<ScanComponentSelector>,

    /// In DCT modes of operation, this parameter specifies the first DCT coefficient in each block
    /// in zig-zag order which shall be coded in the scan. This parameter is set to zero for
    /// sequential DCT processes. In the lossless mode of operations this parameter is used to
    /// select the predictor
    pub(crate) predictor_selection: u8,

    /// Specifies the last DCT coefficient in each block in zig-zag order which shall be coded in
    /// the scan. This parameter shall be set to 63 for sequential DCT processes. In the lossless
    /// mode of operations this parameter has no meaning. It shall be set to zero.
    pub(crate) end_of_spectral_selection: u8,

    /// This parameter specifies the point transform used in the preceding scan (the successive
    /// approximation bit position low in the preceding scan) for the band of coefficients specified
    /// by `predictor_selection` and `end_of_spectral_selection`. This parameter shall be set to
    /// zero for the first scan of each band of coefficients. In the lossless mode of operations
    /// this parameter has no meaning. It shall be set to zero.
    pub(crate) successive_approx_bit_position_high: u8,

    /// In DCT modes of operation, this parameter specifies the point transform (bit position low,
    /// used before coding the band of coefficients specified by `predictor_selection` and
    /// `end_of_spectral_selection`. This parameter shall be set to zero for sequential DCT processes.
    /// In the lossless mode of operations, this parameter specifies the point transform Pt.
    pub(crate) point_transform: u8,
}

#[derive(Debug)]
pub struct ScanComponentSelector {
    /// Selects which of the component type specified in the frame parameters shall be the jth
    /// component in the scan. Each scan component selector shall match one of the Ci values
    /// specified in the frame header.
    component_id: u8,

    /// Specifies one of four possible DC entropy coding table destinations from which the entropy
    /// table needed for decoding of the DC coefficients of component selector j is retrieved.
    dc_destination_id: u8,

    /// Specifies one of four possible AC entropy coding table destinations from which the entropy
    /// table needed for decoding of the AC coefficients of component selector j is retrieved.
    ac_destination_id: u8,
}

impl ScanComponentSelector {
    pub(crate) fn from(component_id: u8, dc_destination_id: u8, ac_destination_id: u8) -> Self {
        Self {
            component_id,
            dc_destination_id,
            ac_destination_id
        }
    }
}

