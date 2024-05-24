use anyhow::Result;

pub struct Header {
    version: f32,
    units: usize,
    density: (usize, usize),
    thumbnail: (usize, usize),
}

pub struct JsonDecoder {
    buffer: Vec<u8>
}

impl JsonDecoder {
    pub fn new(
        buffer: &[u8]
    ) -> Self {
        JsonDecoder {
            buffer: buffer.to_vec()
        }
    }
    pub fn decode(&self) -> Result<()> {

        Ok(())
    }
}