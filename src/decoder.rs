use crate::bitreader::BitReader;
use crate::coding::{CodingProcess, EntropyCoding};
use crate::entropy_decoder::EntropyDecoder;
use crate::huffman_tree::HuffmanClass;
use crate::marker::{Marker, MarkerType};
use crate::parser::Parser;
use crate::sample_precision::SamplePrecision;
use anyhow::{anyhow, Result};
use memmap::Mmap;
use rayon::iter::IntoParallelRefIterator;
use rayon::iter::ParallelIterator;
use std::collections::HashMap;
use std::fs::File;
use std::simd::prelude::*;

type Marlen = (usize, usize); // offset, length

pub struct Decoder {
    pub(crate) mmap: Mmap,
    pub(crate) cursor: usize,
    pub(crate) encoding: CodingProcess,
}

impl Decoder {
    const LANE_COUNT: usize = 64;

    pub fn from_file(file: File) -> Result<Self> {
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(Decoder {
            mmap,
            cursor: 0,
            encoding: CodingProcess::BaselineDCT,
        })
    }

    pub fn from_file_path(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        Decoder::from_file(file)
    }

    fn check_start_of_image(&mut self) -> Result<()> {
        let start: Simd<u8, 2> =
            Simd::from_array([self.mmap[self.cursor], self.mmap[self.cursor + 1]]);

        match start
            .simd_eq(Simd::from([Marker::GLOBAL as u8, Marker::SOI as u8]))
            .all()
        {
            true => Ok(()),
            false => Err(anyhow!("Error, failed to find SOI marker")),
        }
    }

    fn scan_markers(&mut self) -> Result<HashMap<Marker, Vec<Marlen>>> {
        let mut temp_chunk = [0u8; Self::LANE_COUNT];
        let mut all_markers = Marker::all();
        let mut marker_marlen_map = HashMap::new();

        while self.cursor < self.mmap.len() {
            let end = (self.cursor + Self::LANE_COUNT).min(self.mmap.len());
            let len = end - self.cursor;

            temp_chunk[..len].copy_from_slice(&self.mmap[self.cursor..end]);
            let mut curr_chunk = Simd::from_array(temp_chunk);

            let high_marker_mask = Simd::splat(Marker::GLOBAL as u8);
            let high_marker_matches = curr_chunk.simd_eq(high_marker_mask);
            if !high_marker_matches.any() {
                temp_chunk = [0u8; Self::LANE_COUNT];
                self.cursor += Self::LANE_COUNT;

                continue;
            }

            curr_chunk = curr_chunk.rotate_elements_left::<1>();

            let visited_markers: Vec<_> = all_markers
                .par_iter()
                .filter_map(|low_marker| {
                    let low_marker_mask = Simd::splat(*low_marker as u8);
                    let low_marker_matches = curr_chunk.simd_eq(low_marker_mask);
                    if !low_marker_matches.any() {
                        return None;
                    }

                    let mut marker_matches = high_marker_matches & low_marker_matches;
                    if !marker_matches.any() {
                        return None;
                    }

                    let mut local_visited_markers = vec![];
                    let mut local_marker_marlen_map = vec![];

                    while let Some(marker_index) = marker_matches.first_set() {
                        marker_matches.set(marker_index, false);

                        if low_marker.singleton() {
                            local_visited_markers.push(*low_marker);
                        }

                        let marker_offset = self.cursor + marker_index;
                        let segment_offset = marker_offset + Marker::SIZE;

                        /*
                        A note about marlen. It contains the offset of the [segment].
                        Consider the two cases:

                        Segment Marker:
                        [SOF][Length][Segment .... (length) ....]
                                     | <- this is the marlen position

                        Standalone Marker:
                        [SOI][....]
                             | <- this is the marlen position

                         */
                        let segment_marlen = match low_marker.is_segment() {
                            MarkerType::Segment => (
                                segment_offset + 2,
                                u16::from_be_bytes([
                                    self.mmap[segment_offset],
                                    self.mmap[segment_offset + 1],
                                ]) as usize
                                    - 2,
                            ),
                            MarkerType::StandAlone => (segment_offset, 0),
                        };

                        local_marker_marlen_map.push((*low_marker, segment_marlen))
                    }

                    Some((local_visited_markers, local_marker_marlen_map))
                })
                .collect();

            for (local_visited_markers, local_marker_marlen_map) in visited_markers {
                for marker in local_visited_markers {
                    all_markers.remove(&marker);
                }

                for (marker, segment_marlen) in local_marker_marlen_map {
                    marker_marlen_map
                        .entry(marker)
                        .or_insert_with(Vec::new)
                        .push(segment_marlen)
                }
            }

            temp_chunk = [0u8; Self::LANE_COUNT];
            self.cursor += Self::LANE_COUNT;
        }

        Ok(marker_marlen_map)
    }

    pub(crate) fn setup(&mut self) -> Result<Parser> {
        self.check_start_of_image()?;
        let marlen_map = self.scan_markers()?;

        Ok(Parser::new(self.mmap.to_vec(), marlen_map, self.encoding))
    }

    pub fn decode(&mut self) -> Result<()> {
        let parser = self.setup()?;

        let code_schema = self.encoding.schema();

        match self.encoding {
            CodingProcess::BaselineDCT => {
                let huffman_trees = parser.parse_huffman_trees()?;
                let quantization_tables = parser.parse_quant_table()?;
                let frame_header = parser.parse_start_of_frame()?;
                let (scan_header, encoded_image_start_index) = parser.parse_start_of_scan()?;
                let compressed_image_data = parser.parse_image_data(encoded_image_start_index)?;

                // validation....
                if frame_header.component_type != scan_header.component_type {
                    return Err(anyhow!("header component types do not align."));
                }

                let (num_ac_tables, num_dc_tables) =
                    huffman_trees
                        .iter()
                        .fold((0, 0), |(ac_count, dc_count), ht| match ht.class {
                            HuffmanClass::AC => (ac_count + 1, dc_count),
                            HuffmanClass::DC => (ac_count, dc_count + 1),
                        });

                let (expected_ac_tables, expected_dc_tables) = code_schema.entropy_table_count;
                if num_ac_tables != expected_ac_tables || num_dc_tables != expected_dc_tables {
                    return Err(anyhow!(
                        "number of ac & dc entropy tables mismatch from expected."
                    ));
                }

                let precisions: Vec<SamplePrecision> =
                    quantization_tables.iter().map(|qt| qt.precision).collect();

                if !precisions.iter().all(|p| *p == SamplePrecision::EightBit) {
                    return Err(anyhow!(format!(
                        "expected 8-bit samples within each component. Got {:?}",
                        &precisions
                    )));
                }

                // now commence the bit stream
                let mut bit_reader = BitReader::new(&compressed_image_data);
                let bits = bit_reader.slice_to_bits();

                let mut entropy_decoder = EntropyDecoder::new(
                    &bits,
                    scan_header,
                    frame_header,
                    EntropyCoding::Huffman(huffman_trees),
                );

                let _decompressed_image_data = entropy_decoder.decode()?;
            }
            _ => todo!(),
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode() -> Result<()> {
        let mut decoder = Decoder {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
            encoding: CodingProcess::BaselineDCT,
        };

        decoder.decode()?;

        Ok(())
    }
}
