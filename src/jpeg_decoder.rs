use crate::component::{Component, ComponentType, FrameData, ScanData};
use crate::huffman_tree::{CodeFreq, HuffmanTree};
use crate::image::Image;
use crate::jfif_reader::{MarLen, MARKER_BYTES};
use crate::quant_tables::{Precision, QuantTable};
use anyhow::{anyhow, Result};
use std::iter;
use std::simd::prelude::*;
use std::simd::LaneCount;

const INFORMATION_BYTES: usize = 1;
const HUFFMAN_SYM_BYTES: usize = 16;

pub const QUANTIZATION_TABLE_BYTES: usize = 64;

pub struct JpegDecoder {
    buffer: Vec<u8>,
    huffman_marlen: Vec<MarLen>,
    qt_marlen: Vec<MarLen>,
    sos_marlen: MarLen,
    sof_marlen: MarLen,
}

impl JpegDecoder {
    pub fn new(
        buffer: &[u8],
        huffman_marlen: Vec<MarLen>,
        qt_marlen: Vec<MarLen>,
        sos_marlen: MarLen,
        sof_marlen: MarLen,
    ) -> Self {
        JpegDecoder {
            buffer: buffer.to_vec(),
            huffman_marlen,
            qt_marlen,
            sos_marlen,
            sof_marlen,
        }
    }

    pub fn decode(&self) -> Result<Image> {
        let huffman_trees = self.decode_huffman_trees()?;
        let quant_tables = self.decode_quant_table()?;
        let start_of_frame = self.decode_start_of_frame()?;
        let (start_of_scan, start_of_image_data_index) = self.decode_start_of_scan()?;

        println!("image data without byte stuffing: {}, entire length of data: {}", self.buffer.len() - start_of_image_data_index, self.buffer.len());

        let image_data = self.sanitize_image_data(start_of_image_data_index)?;

        Ok(Image {
            data: image_data,
            huffman_trees,
            quant_tables,
            start_of_frame,
            start_of_scan,
        })
    }

    fn decode_huffman_information(&self) -> Result<([u8; 4], [u8; 4])> {
        let ht_informations: Simd<u8, 4> = Simd::from_slice(
            &self
                .huffman_marlen
                .iter()
                .map(|marlen| self.buffer[marlen.offset])
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
        let qt_informations: Simd<u8, 2> = Simd::from_slice(
            &self
                .qt_marlen
                .iter()
                .map(|marlen| self.buffer[marlen.offset])
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
        debug_assert_eq!(self.qt_marlen.len(), 2);

        let mut tables = vec![];

        let (qt_ids, qt_precisions) = self.decode_quant_table_information()?;

        for (idx, marlen) in self.qt_marlen.iter().enumerate() {
            let MarLen { offset, length } = marlen;

            let mut current_offset = offset + MARKER_BYTES;
            debug_assert!(self.buffer.len() > current_offset + QUANTIZATION_TABLE_BYTES);

            let qt_data: Simd<u8, QUANTIZATION_TABLE_BYTES> = Simd::from_slice(
                &self.buffer[current_offset..current_offset + QUANTIZATION_TABLE_BYTES],
            );

            let (qt_id, qt_precision) = (qt_ids[idx], qt_precisions[idx]);
            tables.push(QuantTable::from(qt_id, qt_precision, qt_data))
        }

        Ok(tables)
    }

    fn decode_huffman_trees(&self) -> Result<Vec<HuffmanTree>> {
        debug_assert_eq!(self.huffman_marlen.len(), 4);

        let mut trees = vec![];

        let (ht_types, ht_numbers) = self.decode_huffman_information()?;

        for (idx, marlen) in self.huffman_marlen.iter().enumerate() {
            let MarLen { offset, length } = marlen;

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
                .map(|(&code, &freq)| CodeFreq { code, freq })
                .collect::<Vec<CodeFreq>>();

            let tree = HuffmanTree::from(ht_types[idx], ht_numbers[idx] as usize, code_freq);
            trees.push(tree);
        }

        Ok(trees)
    }

    fn decode_start_of_scan(&self) -> Result<(Vec<ScanData>, usize)> {
        let MarLen { offset, .. } = self.sos_marlen;
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
        let MarLen { offset, length } = self.sof_marlen;
        let mut current_offset = offset;

        let precision = Precision::parse(self.buffer[current_offset]);
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
        let end_of_image_data_index = self.buffer.len() - MARKER_BYTES - 1;
        let image_length = end_of_image_data_index - start_of_image_data_index;

        let mut current_index = start_of_image_data_index;
        const LANE_COUNT: usize = 64;

        let mut temp_chunk = [0u8; LANE_COUNT];
        let mut result = Vec::with_capacity(image_length);

        while current_index < self.buffer.len() - MARKER_BYTES {
            let end = (current_index + LANE_COUNT).min(self.buffer.len() - MARKER_BYTES);
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
    use crate::jfif_reader::JFIFReader;
    use memmap::Mmap;
    use std::fs::{File, OpenOptions};
    use std::io::Write;
    use std::sync::Once;

    fn mike_decoder() -> Result<JpegDecoder> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        Ok(jfif_reader.decoder()?)
    }

    #[test]
    fn test_decode_mike() -> Result<()> {
        let decoder = mike_decoder()?;
        let huffman_trees = decoder.decode_huffman_trees()?;
        let FrameData { image_width, image_height, .. } = decoder.decode_start_of_frame()?;

        let qt_tables = decoder.decode_quant_table()?;

        assert_eq!(image_width, 640);
        assert_eq!(image_height, 763);
        assert_eq!(qt_tables.len(), 2);

        Ok(())
    }

    static INIT: Once = Once::new();

    fn setup() {
        INIT.call_once(|| {
            let data = vec![
                0xFF, 0xDA,
            ];

            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("mock_jpeg_data_decode.bin")
                .unwrap();
            file.write_all(&data).unwrap();
        });
    }



}
