use crate::interchange::marker::{Marker, MarkerType};
use anyhow::{anyhow, Result};
use memmap::Mmap;
use rayon::iter::*;
use std::collections::HashMap;
use std::fs::File;
use std::simd::prelude::*;
use crate::decoder::baseline_process::decoder::JpegDecoder;
use crate::interchange::Compression;

pub struct JFIFReader {
    pub mmap: Mmap,
    pub cursor: usize,
}

impl JFIFReader {
    const LANE_COUNT: usize = 64;

    pub fn from_file(file: File) -> Result<Self> {
        let mmap = unsafe { Mmap::map(&file)? };
        Ok(JFIFReader { mmap, cursor: 0 })
    }

    pub fn from_file_path(file_path: &str) -> Result<Self> {
        let file = File::open(file_path)?;
        JFIFReader::from_file(file)
    }

    pub(crate) fn decoder(&mut self, compression: Compression) -> Result<JpegDecoder> {
        let marker_marlen_map = self.scan_markers()?;

        /*
        [SOI][     FRAME     ][EOI]
         */

        if !marker_marlen_map.contains_key(&Marker::SOI)
            || !marker_marlen_map.contains_key(&Marker::EOI)
        {
            return Err(anyhow!(
                "Start of Image markery and End of Marker image must be in image data"
            ));
        }

        let soi_marlen = marker_marlen_map
            .get(&Marker::SOI)
            .ok_or(anyhow!("failed to find soi marker"))?;
        let eoi_marlen = marker_marlen_map
            .get(&Marker::EOI)
            .ok_or(anyhow!("failed to find eoi marker"))?;

        if soi_marlen.len() != 1 || eoi_marlen.len() != 1 {
            return Err(anyhow!(
                "Found marker, but was not able to find any marker-lengths"
            ));
        }

        let (soi_marlen_offset, soi_marlen_length) = soi_marlen[0];
        let (eoi_marlen_offset, eoi_marlen_length) = eoi_marlen[0];

        debug_assert_eq!(soi_marlen_length, 0);
        debug_assert_eq!(eoi_marlen_length, 0);

        if soi_marlen_offset != Marker::SIZE {
            return Err(anyhow!(
                "Start of Image marlen should be immediately after the marker bytes in image data"
            ));
        }

        if eoi_marlen_offset != self.mmap.len() {
            return Err(anyhow!(
                "End of Image marlen should be the first out of bounds index in image data"
            ));
        }

        Ok(JpegDecoder::new(self.mmap.to_vec(), marker_marlen_map))
    }

    fn scan_markers(&mut self) -> Result<HashMap<Marker, Vec<(usize, usize)>>> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::OpenOptions;
    #[test]
    fn validate_mike() -> Result<()> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        let decoder = jfif_reader.decoder(Compression::Baseline);
        assert!(decoder.is_ok());

        Ok(())
    }

    #[test]
    fn validate_mock() -> Result<()> {
        let file = File::open("mock_jpeg_decode.bin")?;
        let mmap = unsafe { Mmap::map(&file)? };

        let mut jfif_reader = JFIFReader { mmap, cursor: 0 };

        let marlen_map = jfif_reader.scan_markers()?;

        println!("marlen map: {:?}", marlen_map);

        Ok(())
    }
}
