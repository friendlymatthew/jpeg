use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::iter;
use std::simd::prelude::*;
use crate::interchange::jfif::JFIF;
use crate::interchange::component::{Component, ComponentType, FrameData, ScanData};
use crate::entropy::EntropyCoding;
use crate::entropy::huffman_table::HuffmanTree;
use crate::interchange::marker::Marker;
use crate::interchange::sample_precision::SamplePrecision;
use crate::quantize::quantization_table::QuantTable;

const INFORMATION_BYTES: usize = 1;
const HUFFMAN_SYM_BYTES: usize = 16;

pub const QUANTIZATION_TABLE_BYTES: usize = 64;

pub(crate) type Marlen = (usize, usize); // offset, length
pub(crate) type MarlenMap = HashMap<Marker, Vec<Marlen>>;

pub(crate) struct JpegDecoder {
    buffer: Vec<u8>,
    marlen_map: MarlenMap,
}

impl JpegDecoder {
    pub fn new(buffer: Vec<u8>, marlen_map: MarlenMap) -> Self {
        JpegDecoder { buffer, marlen_map }
    }

    pub fn decode(&self) -> Result<JFIF> {
        let jfif = self.decode_jfif()?;

        /*
        The entropy decoder decodes the zig zag sequence of quantized dct coefficients.
        After dequantization the DCT coefficients are transformed to an 8 x 8 block of samples by
        the inverse DCT (IDCT).
         */

        Ok(jfif)
    }

    pub fn decode_jfif(&self) -> Result<JFIF> {
        let entropy_coding= self.decode_huffman_trees()?;
        let quant_tables = self.decode_quant_table()?;
        let frame_header= self.decode_start_of_frame()?;
        let (scan_header, start_of_image_data_index) = self.decode_start_of_scan()?;

        println!(
            "image data without byte stuffing: {}, entire length of data: {}",
            self.buffer.len() - start_of_image_data_index,
            self.buffer.len()
        );

        let image_data = self.sanitize_image_data(start_of_image_data_index)?;

        Ok(JFIF {
            data: image_data,
            entropy_coding: EntropyCoding::Huffman(entropy_coding),
            quant_tables,
            frame_header,
            scan_header
        })
    }

    fn decode_huffman_information(&self) -> Result<([u8; 4], [u8; 4])> {
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

    fn decode_quant_table_information(&self) -> Result<([u8; 2], [u8; 2])> {
        let qt_marlens = self.get_marker_segment(&Marker::DQT)?;
        debug_assert_eq!(qt_marlens.len(), 2);

        let qt_informations: Simd<u8, 2> = Simd::from_slice(
            &qt_marlens
                .iter()
                .map(|(o, _)| self.buffer[*o])
                .collect::<Vec<u8>>(),
        );

        // extract ht information
        let qt_precisions_mask = Simd::splat(0b1111);
        let qt_precisions = qt_informations & qt_precisions_mask;

        let qt_ids_mask = Simd::splat(0b11110000);
        let qt_ids = (qt_informations & qt_ids_mask) >> 4;

        let qt_precisions = qt_precisions.to_array();
        let qt_ids = qt_ids.to_array();

        Ok((qt_ids, qt_precisions))
    }

    fn decode_quant_table(&self) -> Result<Vec<QuantTable>> {
        let mut tables = vec![];

        let (qt_ids, qt_precisions) = self.decode_quant_table_information()?;

        let qt_marlens = self.get_marker_segment(&Marker::DQT)?;
        for (idx, (offset, _)) in qt_marlens.iter().enumerate() {
            let current_offset = offset + Marker::SIZE;
            debug_assert!(self.buffer.len() > current_offset + QUANTIZATION_TABLE_BYTES);

            let qt_data: Simd<u8, QUANTIZATION_TABLE_BYTES> = Simd::from_slice(
                &self.buffer[current_offset..current_offset + QUANTIZATION_TABLE_BYTES],
            );

            let (qt_id, qt_precision) = (qt_ids[idx], qt_precisions[idx]);
            tables.push(QuantTable::from(qt_id, qt_precision, qt_data))
        }

        Ok(tables)
    }

    fn get_marker_segment(&self, marker: &Marker) -> Result<&Vec<(usize, usize)>> {
        Ok(self
            .marlen_map
            .get(marker)
            .ok_or(anyhow!("failed to get marker"))?)
    }

    fn decode_huffman_trees(&self) -> Result<Vec<HuffmanTree>> {
        let huffman_marlens = self.get_marker_segment(&Marker::DHT)?;
        debug_assert_eq!(huffman_marlens.len(), 4);

        let mut trees = vec![];

        let (ht_types, ht_numbers) = self.decode_huffman_information()?;

        for (idx, (offset, length)) in huffman_marlens.iter().enumerate() {
            let mut current_offset = offset + INFORMATION_BYTES;

            if self.buffer.len() < current_offset + HUFFMAN_SYM_BYTES {
                return Err(anyhow!("Not enough data to extract symbol table"));
            }

            let sym_table = &self.buffer[current_offset..current_offset + HUFFMAN_SYM_BYTES];

            let mut flat_lengths = vec![];

            for (idx, mult) in sym_table.iter().enumerate() {
                flat_lengths.extend(iter::repeat(idx + 1).take(*mult as usize));
            }

            current_offset += HUFFMAN_SYM_BYTES;

            let code_len = (offset + length) - current_offset;
            debug_assert_eq!(current_offset + code_len, offset + length);

            let code_freq = self.buffer[current_offset..current_offset + code_len]
                .iter()
                .zip(flat_lengths.iter())
                .map(|(&code, &freq)| (code, freq))
                .collect::<Vec<_>>();

            let tree = HuffmanTree::from(ht_types[idx], ht_numbers[idx] as usize, code_freq);
            trees.push(tree);
        }

        Ok(trees)
    }

    fn decode_start_of_scan(&self) -> Result<(Vec<ScanData>, usize)> {
        let sos_marlens = self.get_marker_segment(&Marker::SOS)?;
        debug_assert_eq!(sos_marlens.len(), 1);

        let (offset, _) = sos_marlens[0];

        let mut current_offset = offset;

        let num_components = self.buffer[current_offset];
        current_offset += 1;

        debug_assert_eq!(
            num_components, 3,
            "as of now assume only dealing with color components is 3"
        );

        let mut scan_data = vec![];

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
            scan_data.push(ScanData::from(
                component_ids[i],
                dc_huffman_table_ids[i],
                ac_huffman_table_ids[i],
            ));
        }

        current_offset += 2 * (num_components as usize);
        // always skip 3 bytes.
        current_offset += 3;

        Ok((scan_data, current_offset))
    }

    fn decode_start_of_frame(&self) -> Result<FrameData> {
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

        let num_components = ComponentType::from(self.buffer[current_offset]);
        current_offset += 1;

        let mut components = vec![];

        match num_components {
            ComponentType::Grayscale => {
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
            ComponentType::Color => {
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

        Ok(FrameData {
            precision,
            image_height,
            image_width,
            component_type: num_components,
            components,
        })
    }

    fn sanitize_image_data(&self, start_of_image_data_index: usize) -> Result<Vec<u8>> {
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
    use memmap::Mmap;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::sync::Once;
    use crate::entropy::huffman_table::HuffmanClass;
    use crate::interchange::Compression;
    use crate::interchange::reader::JFIFReader;

    fn mike_decoder() -> Result<JpegDecoder> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        Ok(jfif_reader.decoder(Compression::Baseline)?)
    }

    #[test]
    fn test_decode_mike() -> Result<()> {
        let decoder = mike_decoder()?;
        let _huffman_trees = decoder.decode_huffman_trees()?;
        let FrameData {
            image_width,
            image_height,
            ..
        } = decoder.decode_start_of_frame()?;

        let qt_tables = decoder.decode_quant_table()?;

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
                0x00, 0x0C, 0x03, 0x01, 0x00, 0x02, 0x11, 0x03, 0x11, 0x00, 0x3F,
                0x00, // three bytes that we skip in sos
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

        let mut jpeg_reader = JFIFReader { mmap, cursor: 0 };
        let jfif = jpeg_reader.decoder(Compression::Baseline)?.decode()?;

        let FrameData {
            precision,
            image_height,
            image_width,
            component_type,
            components,
        } = jfif.frame_header;
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

        let huffman_trees = match jfif.entropy_coding {
            EntropyCoding::Huffman(ht) => ht,
            _ => unreachable!()
        };

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

        assert_eq!(
            jfif.data,
            [0xFF, 0x00, 0xFF, 0xFF, 0x02, 0x04, b'h', 0x02,].to_vec()
        );

        Ok(())
    }
}
