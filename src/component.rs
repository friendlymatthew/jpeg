use crate::quant_tables::Precision;

#[derive(Debug)]
pub struct FrameData {
    pub(crate) precision: Precision,
    pub(crate) image_height: usize, // in pixels
    pub(crate) image_width: usize,  //
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

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug)]
pub(crate) struct Component {
    component_id: u8,
    horizontal_scaling_factor: u8,
    vertical_scaling_factor: u8,
    qt_table_id: u8,
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
