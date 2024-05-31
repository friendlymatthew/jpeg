use std::collections::HashMap;
use std::fs::File;
use std::simd::prelude::*;
use memmap::Mmap;
use anyhow::{Result};
use rayon::iter::*;
use crate::marker::{Marker, MarkerType};

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

    fn read(&mut self) {}

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

            let mut visited_markers: Vec<_> = all_markers.par_iter().filter_map(|low_marker| {
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
                    [SOF][Segment .... (length) ....]
                         | <- this is the marlen position

                    Standalone Marker:
                    [SOI][....]
                         | <- this is the marlen position

                     */
                    let segment_marlen= (segment_offset, match low_marker.is_segment() {
                        MarkerType::Segment => {
                            u16::from_be_bytes([self.mmap[segment_offset], self.mmap[segment_offset+ 1]]) as usize
                        },
                        MarkerType::StandAlone => 0,
                    });

                    local_marker_marlen_map.push((*low_marker, segment_marlen))
                }

                Some((local_visited_markers, local_marker_marlen_map))
            }).collect();

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
    #[test]
    fn validate_soi_eoi() -> Result<()> {
        let mut jfif_reader = JFIFReader {
            mmap: unsafe { Mmap::map(&File::open("mike.jpg")?)? },
            cursor: 0,
        };

        let map = jfif_reader.scan_markers()?;

        println!("{:?}", map);

        Ok(())
    }
}