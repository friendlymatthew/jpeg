use crate::coding::CodingProcess;
use crate::frame_header::{Component, ComponentType, FrameHeader};
use crate::huffman_tree::HuffmanTree;
use crate::marker::Marker;
use crate::quantization_table::QuantizationTable;
use crate::sample_precision::SamplePrecision;
use crate::scan_header::{EncodingOrder, ScanComponentSelector, ScanHeader};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::iter;
use std::simd::prelude::*;

pub const QUANTIZATION_TABLE_BYTES: usize = 64;

pub(crate) type Marlen = (usize, usize); // offset, length
pub(crate) type MarlenMap = HashMap<Marker, Vec<Marlen>>;

pub(crate) struct Parser {
    buffer: Vec<u8>,
    marlen_map: MarlenMap,
    encoding: CodingProcess,
}

impl Parser {
    pub fn new(buffer: Vec<u8>, marlen_map: MarlenMap, encoding: CodingProcess) -> Self {
        Parser {
            buffer,
            marlen_map,
            encoding,
        }
    }

    fn parse_huffman_information(&self) -> Result<([u8; 4], [u8; 4])> {
        let huffman_marlen = self.get_marker_segment(&Marker::DHT)?;

        let ht_informations: Simd<u8, 4> = Simd::from_slice(
            &huffman_marlen
                .iter()
                .map(|(o, _)| self.buffer[*o])
                .collect::<Vec<u8>>(),
        );

        // extract ht information
        let ht_number_mask = Simd::splat(0b1111);
        let ht_numbers = ht_informations & ht_number_mask;

        // extract ht type (bit 4)
        let ht_type_mask = Simd::splat(0b10000);
        let ht_types = (ht_informations & ht_type_mask) >> 4;

        let ht_numbers = ht_numbers.to_array();
        let ht_types = ht_types.to_array();

        Ok((ht_types, ht_numbers))
    }

    fn parse_quant_table_information(&self) -> Result<([u8; 2], [u8; 2])> {
        let qt_marlens = self.get_marker_segment(&Marker::DQT)?;
        debug_assert_eq!(qt_marlens.len(), 2);

        let qt_informations: Simd<u8, 2> = Simd::from_slice(
            &qt_marlens
                .iter()
                .map(|(o, _)| self.buffer[*o])
                .collect::<Vec<u8>>(),
        );

        // extract ht information
        let qt_precisions_mask = Simd::splat(0b11110000);
        let qt_precisions = qt_informations & qt_precisions_mask;

        let qt_ids_mask = Simd::splat(0b1111);
        let qt_ids = (qt_informations & qt_ids_mask) >> 4;

        let qt_precisions = qt_precisions.to_array();
        let qt_ids = qt_ids.to_array();

        Ok((qt_ids, qt_precisions))
    }

    pub(crate) fn parse_quant_table(&self) -> Result<Vec<QuantizationTable>> {
        let mut tables = vec![];

        let (qt_ids, qt_precisions) = self.parse_quant_table_information()?;

        let qt_marlens = self.get_marker_segment(&Marker::DQT)?;
        for (idx, (offset, _)) in qt_marlens.iter().enumerate() {
            let current_offset = offset + Marker::SIZE;
            debug_assert!(self.buffer.len() > current_offset + QUANTIZATION_TABLE_BYTES);

            let qt_data: Simd<u8, QUANTIZATION_TABLE_BYTES> = Simd::from_slice(
                &self.buffer[current_offset..current_offset + QUANTIZATION_TABLE_BYTES],
            );

            let (qt_id, qt_precision) = (qt_ids[idx], qt_precisions[idx]);
            tables.push(QuantizationTable::from(qt_id, qt_precision, qt_data))
        }

        Ok(tables)
    }

    fn get_marker_segment(&self, marker: &Marker) -> Result<&Vec<(usize, usize)>> {
        Ok(self
            .marlen_map
            .get(marker)
            .ok_or(anyhow!("failed to get marker"))?)
    }

    pub(crate) fn parse_huffman_trees(&self) -> Result<Vec<HuffmanTree>> {
        let huffman_marlens = self.get_marker_segment(&Marker::DHT)?;
        debug_assert_eq!(huffman_marlens.len(), 4);

        let mut trees = vec![];

        let (ht_types, ht_numbers) = self.parse_huffman_information()?;

        for (idx, (offset, length)) in huffman_marlens.iter().enumerate() {
            let mut current_offset = offset + 1;

            if self.buffer.len() < current_offset + 16 {
                return Err(anyhow!("Not enough data to extract symbol table"));
            }

            let sym_table = &self.buffer[current_offset..current_offset + 16];

            let mut flat_lengths = vec![];

            for (idx, mult) in sym_table.iter().enumerate() {
                flat_lengths.extend(iter::repeat(idx + 1).take(*mult as usize));
            }

            current_offset += 16;

            let code_len = (offset + length) - current_offset;
            debug_assert_eq!(current_offset + code_len, offset + length);

            let code_freq = self.buffer[current_offset..current_offset + code_len]
                .iter()
                .zip(flat_lengths.iter())
                .map(|(&code, &freq)| (code, freq))
                .collect::<Vec<_>>();

            let tree = HuffmanTree::from(ht_types[idx], ht_numbers[idx], code_freq);
            trees.push(tree);
        }

        Ok(trees)
    }

    pub(crate) fn parse_start_of_scan(&self) -> Result<(ScanHeader, usize)> {
        let sos_marlens = self.get_marker_segment(&Marker::SOS)?;
        debug_assert_eq!(sos_marlens.len(), 1);

        let (offset, _) = sos_marlens[0];

        let mut current_offset = offset;

        let (component_type, encoding_order) = ComponentType::from(self.buffer[current_offset]);
        current_offset += 1;

        debug_assert_eq!(
            component_type,
            ComponentType::Color,
            "as of now assume only dealing with color components is 3"
        );

        let mut scan_component_selectors = vec![];

        let component_ids = Simd::from([
            self.buffer[current_offset],
            self.buffer[current_offset + 2],
            self.buffer[current_offset + (2 * 2)],
            0,
        ]);

        current_offset += 1;

        let huffman_table_ids = Simd::from([
            self.buffer[current_offset],
            self.buffer[current_offset + 2],
            self.buffer[current_offset + (2 * 2)],
            0,
        ]);

        current_offset -= 1;

        let dc_huffman_table_ids = huffman_table_ids >> 4;
        let ac_huffman_table_ids = huffman_table_ids & Simd::splat(0b1111);

        for i in 0..3 {
            scan_component_selectors.push(ScanComponentSelector::from(
                component_ids[i],
                dc_huffman_table_ids[i],
                ac_huffman_table_ids[i],
            ));
        }

        current_offset += 2 * (component_type as usize);

        let start_of_spectral = self.buffer[current_offset];
        current_offset += 1;

        let end_of_spectral = self.buffer[current_offset];
        current_offset += 1;

        let approx_bit_chunk = self.buffer[current_offset];
        current_offset += 1;

        let (successive_approx_bit_position_high, point_transform) =
            (approx_bit_chunk >> 4, approx_bit_chunk & 0b1111);

        Ok((
            ScanHeader {
                encoding_order,
                component_type,
                scan_component_selectors,
                start_of_spectral,
                end_of_spectral,
                successive_approx_bit_position_high,
                point_transform,
            },
            current_offset,
        ))
    }

    pub(crate) fn parse_start_of_frame(&self) -> Result<FrameHeader> {
        let sof_marlens = self.get_marker_segment(&Marker::SOF0)?;
        debug_assert_eq!(sof_marlens.len(), 1);

        let (offset, _) = sof_marlens[0];
        let mut current_offset = offset;

        let precision = SamplePrecision::parse(self.buffer[current_offset]);
        current_offset += 1;

        let image_dim: Simd<u8, 4> =
            Simd::from_slice(&self.buffer[current_offset..current_offset + 4]);
        let (image_height, image_width) = (
            (((image_dim[0] as u16) << 8) | (image_dim[1] as u16)) as usize,
            (((image_dim[2] as u16) << 8) | (image_dim[3] as u16)) as usize,
        );

        current_offset += 4;

        let component_type = ComponentType::from(self.buffer[current_offset]);
        current_offset += 1;

        let mut components = vec![];

        match component_type.1 {
            EncodingOrder::NonInterleaved => {
                // naive solution
                let component_id = self.buffer[current_offset];
                current_offset += 1;
                let sampling_factor = self.buffer[current_offset];
                let (horizontal_factor, vertical_factor) =
                    (sampling_factor >> 4, sampling_factor & 0b1111);
                current_offset += 1;
                let qt_table_id = self.buffer[current_offset];

                components.push(Component::from(
                    component_id,
                    horizontal_factor,
                    vertical_factor,
                    qt_table_id,
                ))
            }
            EncodingOrder::Interleaved => {
                let component_ids = Simd::from([
                    self.buffer[current_offset],
                    self.buffer[current_offset + 3],
                    self.buffer[current_offset + 2 * 3],
                    0,
                ]);
                current_offset += 1;

                let sampling_factors = Simd::from([
                    self.buffer[current_offset],
                    self.buffer[current_offset + 3],
                    self.buffer[current_offset + 2 * 3],
                    0,
                ]);
                current_offset += 1;

                let qt_table_ids = Simd::from([
                    self.buffer[current_offset],
                    self.buffer[current_offset + 3],
                    self.buffer[current_offset + 2 * 3],
                    0,
                ]);

                let horizontal_factors = sampling_factors >> 4;
                let vertical_factors = sampling_factors & Simd::splat(0b1111);

                for i in 0..3 {
                    let component = Component::from(
                        component_ids[i],
                        horizontal_factors[i],
                        vertical_factors[i],
                        qt_table_ids[i],
                    );
                    components.push(component);
                }
            }
        }

        Ok(FrameHeader {
            precision,
            image_height,
            image_width,
            component_type: component_type.0,
            components,
        })
    }

    pub(crate) fn parse_image_data(&self, start_of_image_data_index: usize) -> Result<Vec<u8>> {
        let end_of_image_data_index = self.buffer.len() - Marker::SIZE - 1;
        let image_length = end_of_image_data_index - start_of_image_data_index;

        let mut current_index = start_of_image_data_index;
        const LANE_COUNT: usize = 64;

        let mut temp_chunk = [0u8; LANE_COUNT];
        let mut result = Vec::with_capacity(image_length);

        while current_index < self.buffer.len() - Marker::SIZE {
            let end = (current_index + LANE_COUNT).min(self.buffer.len() - Marker::SIZE);
            let len = end - current_index;

            temp_chunk[..len].copy_from_slice(&self.buffer[current_index..end]);

            let image_chunk: Simd<u8, LANE_COUNT> = Simd::from_slice(&temp_chunk);
            // suppose i just had [0xFF, 0x00, 0xFF, 0x00]

            let ff_mask = image_chunk.simd_eq(Simd::splat(0xFF));
            // [true, false, true, false]

            let shift_image_chunk = image_chunk.rotate_elements_left::<1>();
            // [0x00, 0xFF, 0x00, 0x00]
            let zero_mask = shift_image_chunk.simd_eq(Simd::splat(0x00));
            // [true, false, true, true]

            let zero_after_ff_mask = ff_mask & zero_mask;
            // [ true, false, true, false]

            let mut chunk_result = Vec::with_capacity(LANE_COUNT);
            let mut i = 0;

            while i < len {
                if zero_after_ff_mask.test(i) {
                    chunk_result.push(temp_chunk[i]);
                    i += 2;
                    continue;
                }
                chunk_result.push(temp_chunk[i]);
                i += 1;
            }

            result.extend(chunk_result);
            current_index += LANE_COUNT;
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoder::Decoder;
    use crate::huffman_tree::HuffmanClass;
    use memmap::Mmap;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::sync::Once;

    fn mike_parser() -> Result<Parser> {
        let mut decoder = Decoder {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
            encoding: CodingProcess::BaselineDCT,
        };

        Ok(decoder.setup()?)
    }

    #[test]
    fn test_parse_mike() -> Result<()> {
        let parser = mike_parser()?;
        let _huffman_trees = parser.parse_huffman_trees()?;
        let FrameHeader {
            image_width,
            image_height,
            ..
        } = parser.parse_start_of_frame()?;

        let qt_tables = parser.parse_quant_table()?;

        assert_eq!(image_width, 640);
        assert_eq!(image_height, 763);
        assert_eq!(qt_tables.len(), 2);

        Ok(())
    }

    static INIT: Once = Once::new();

    // this contains a mock start of frame and start of scan
    pub(crate) fn setup() {
        INIT.call_once(|| {
            let data = vec![
                0xFF, 0xD8, // SOI
                0xFF, 0xE0, // APP0
                0x00, 0x10, b'J', b'F', b'I', b'F', 0x00, 0x01, 0x01, 0x01, 0x00, 0x48, 0x00, 0x48,
                0x00, 0x00, // 16
                0xFF, 0xDB, // QT 1
                0x00, 0x03, 0x00, 0xFF, 0xDB, // QT 2
                0x00, 0x03, 0x00, 0xFF, 0xC0, // START OF FRAME
                0x00, 0x11, 0x08, 0x00, 0x02, 0x00, 0x06, 0x03, 0x01, 0x22, 0x00, 0x02, 0x11, 0x01,
                0x03, 0x11, 0x01, // 17
                0xFF, 0xC4, // HUFFMAN 1 39
                0x00, 0x15, 0x00, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, // 21
                0xFF, 0xC4, // HUFFMAN 2 62
                0x00, 0x19, 0x10, 0x01, 0x00, 0x02, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x06, 0x08, 0x38, 0x88, 0xB6, // 25
                0xFF, 0xC4, // HUFFMAN 3 89
                0x00, 0x15, 0x01, 0x01, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x07, 0x0Aa, // 21
                0xFF, 0xC4, // HUFFMAN 4 112
                0x00, 0x1C, 0x11, 0x00, 0x01, 0x03, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x08, 0x00, 0x07, 0xB8, 0x09, 0x38, 0x39, 0x76,
                0x78, // 28
                0xFF, 0xDA, // START OF SCAN
                0x00, 0x08, 0x03, 0x01, 0x10, 0x01, 0x3F,
                0x10, // three bytes that we skip in sos
                0xFF, // this should be the start of image data
                0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00, 0x02, 0x04, b'h', 0x02, 0xFF, 0xD9, // EOI
            ];

            println!("length of test data: {}", data.len());

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("mock_jpeg_decode.bin")
                .unwrap();
            file.write_all(&data).unwrap();
        });
    }

    #[test]
    fn test_decoding_various_markers() -> Result<()> {
        setup();

        let file = File::open("mock_jpeg_decode.bin")?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut decoder = Decoder {
            mmap,
            cursor: 0,
            encoding: CodingProcess::BaselineDCT,
        };
        let parser = decoder.setup()?;

        let FrameHeader {
            precision,
            image_height,
            image_width,
            component_type,
            components,
        } = parser.parse_start_of_frame()?;
        assert_eq!(precision, SamplePrecision::EightBit);
        assert_eq!(image_width, 6);
        assert_eq!(image_height, 2);
        assert_eq!(component_type, ComponentType::Color);
        assert_eq!(components.len(), 3);
        assert_eq!(
            [
                Component {
                    component_id: 1,
                    horizontal_scaling_factor: 2,
                    vertical_scaling_factor: 2,
                    qt_table_id: 0
                },
                Component {
                    component_id: 2,
                    horizontal_scaling_factor: 1,
                    vertical_scaling_factor: 1,
                    qt_table_id: 1
                },
                Component {
                    component_id: 3,
                    horizontal_scaling_factor: 1,
                    vertical_scaling_factor: 1,
                    qt_table_id: 1
                }
            ]
            .to_vec(),
            components
        );

        let huffman_trees = parser.parse_huffman_trees()?;
        assert_eq!(huffman_trees.len(), 4);
        assert_eq!(
            huffman_trees
                .iter()
                .map(|ht| { ht.class })
                .collect::<Vec<_>>(),
            vec![
                HuffmanClass::DC,
                HuffmanClass::AC,
                HuffmanClass::DC,
                HuffmanClass::AC,
            ]
        );

        assert_eq!(
            huffman_trees
                .iter()
                .map(|ht| { ht.destination_id })
                .collect::<Vec<_>>(),
            vec![0, 0, 1, 1]
        );

        let (scan_header, s_idx) = parser.parse_start_of_scan()?;

        assert_eq!(scan_header.start_of_spectral, 0x01);
        assert_eq!(scan_header.end_of_spectral, 63);
        assert_eq!(scan_header.successive_approx_bit_position_high, 1);
        assert_eq!(scan_header.point_transform, 0);

        assert_eq!(
            parser.parse_image_data(s_idx)?,
            [0xFF, 0x00, 0xFF, 0xFF, 0x02, 0x04, b'h', 0x02,].to_vec()
        );

        Ok(())
    }
}
