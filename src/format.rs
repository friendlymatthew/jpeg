#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Format {
    YCbCr(f32, f32, f32),
    RGB(f32, f32, f32),
}
