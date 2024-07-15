use crate::sample_precision::SamplePrecision;
use crate::scan_header::EncodingOrder;

#[derive(Debug)]
pub struct FrameHeader {
    /// P: Specifies the precision in bits for the samples of the components in the frame
    pub(crate) precision: SamplePrecision,

    /// Y: Number of lines -- Specifies the maximum number of lines in the source image. This shall
    /// be equal to the number of lines in the component with the maximum number of vertical samples.
    pub(crate) image_height: usize,

    /// X: Number of samples per line -- Specifies the maximum number of samples per line in the source
    /// image. This shall be equal to the number of lines the component with the maximum number
    /// of vertical samples.
    pub(crate) image_width: usize,

    /// Nf: Number of image components in frame -- Specifies the number of source image components in
    /// the frame. The value of `num_components` shall be equal to the number of sets of frame
    ///component specification parameters (Ci, Hi, Vi, Tqi) present in the frame header.
    pub(crate) component_type: ComponentType,

    pub(crate) components: Vec<Component>,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum ComponentType {
    Grayscale,
    Color,
}

impl ComponentType {
    pub(crate) fn from(b: u8) -> (Self, EncodingOrder) {
        match b {
            1 => (ComponentType::Grayscale, EncodingOrder::NonInterleaved),
            2 => (ComponentType::Color, EncodingOrder::Interleaved),
            3 => (ComponentType::Color, EncodingOrder::Interleaved),
            4 => todo!(),
            _ => unreachable!(),
        }
    }
}

/// One of the two-dimensional arrays which comprise an image
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Component {
    /// Ci: Assigns a unique label to the ith component in the sequence of frame component specification
    /// parameters. These values shall be used in the scan headers to identify the components in the
    /// scan.
    pub(crate) component_id: u8,

    /// Hi: Specifies the relationship between the component horizontal dimension and `image_width`
    /// ; also specifies the number of horizontal data units of component Ci in each MCU,
    /// when more than one component is encoded in a scan.
    pub(crate) horizontal_scaling_factor: u8,

    /// Vi: Specifies the relationship between the component vertical dimension and `image_height`
    /// ; also specifies the number of vertical data units of component Ci in each MCU, when more
    /// than one component is encoded in a scan.
    pub(crate) vertical_scaling_factor: u8,

    /// Tqi: Specifies one of four possible quantization destinations from which the quantization table
    /// to use for dequantization of DCT coefficients of component Ci is retrieved. If the decoding
    /// process uses the dequantization process, this table shall have been installed in this
    /// destination by the time the decoder is ready to decode the scan(s) containing component Ci.
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
