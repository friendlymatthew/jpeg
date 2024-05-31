use crate::marker::Marker;
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::simd::Simd;

pub struct JFIFDecoder {
    data: Vec<u8>,
    marlen_map: HashMap<Marker, Vec<(usize, usize)>>,
}

impl JFIFDecoder {
    pub fn new(data: Vec<u8>, marlen_map: HashMap<Marker, Vec<(usize, usize)>>) -> Self {
        JFIFDecoder { data, marlen_map }
    }
    pub fn decode(&mut self) -> Result<()> {


        Ok(())
    }



    fn decode_huffman_tables(&mut self) -> Result<()> {
        let dht_marlens = self.marlen_map.get(&Marker::DHT).ok_or(anyhow!("failed to find dht marker in marlen map"))?;
        debug_assert_eq!(dht_marlens.len(), 4, "Expected exactly 4 DHT marker lengths");

        let mut dht_information_data = [0u8; 4];

        let _ = dht_marlens.iter().enumerate().map(
            |(index, (offset, length))| {
                let offset = offset + 2;
                let length = length - 2;

                let data = &self.data[offset..offset + length];
                dht_information_data[index] = data[0];
            }
        );

        let dht_information: Simd<u8, 4> = Simd::from_array(dht_information_data);

        let ht_destination_ids_mask = Simd::splat(0b1111);
        let ht_destination_ids = dht_information & ht_destination_ids_mask;

        let ht_class_mask = Simd::splat(0b10000);
        let ht_class = (dht_information & ht_class_mask) >> 4;



        Ok(())
    }
}
