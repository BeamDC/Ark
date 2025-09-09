use crate::archival::compression::file_compressor::Compressor;

impl Compressor {
    /// run length encoding with one byte of run size
    pub fn rle(&mut self) {
        let mut data = Vec::with_capacity(self.data.len());
        let mut pos = 0;

        while pos < self.data.len() {
            let current = self.data[pos];
            let mut len = 0u8;
            let mut relative_pos = pos;

            // continue until end of run
            while relative_pos < self.data.len() && self.data[relative_pos] == current {
                len += 1;
                relative_pos += 1;
                if len == u8::MAX { break }
            }

            // push run and its data
            data.push(len);
            data.push(current);
            pos = relative_pos;
        }

        self.data = data;
    }

    /// run length encoding with two bytes of run size
    pub fn rle_two_byte(&mut self) {
        let mut data = Vec::with_capacity(self.data.len());
        let mut pos = 0;

        while pos < self.data.len() {
            let current = self.data[pos];
            let mut len = 0u16;
            let mut relative_pos = pos;

            // continue until end of run
            while relative_pos < self.data.len() && self.data[relative_pos] == current {
                len += 1;
                relative_pos += 1;
                if len == u16::MAX { break }
            }

            // push run and its data
            data.push((len >> 8) as u8);
            data.push(len as u8);
            data.push(current);
            pos = relative_pos;
        }

        self.data = data;
    }

    /// decompress from one byte run length encoded data
    pub fn decompress_rle(&mut self) {
        let mut to_decode = self.data.iter().cloned().rev().collect::<Vec<u8>>();
        let mut res = Vec::with_capacity(to_decode.len() * 2);

        while !to_decode.is_empty() {
            let freq = to_decode.pop().unwrap();
            let symbol = to_decode.pop().unwrap();
            res.extend(vec![symbol; freq as usize])
        }

        self.data = res;
    }

    /// decompress from two byte run length encoded data
    pub fn decompress_rle_two_byte(&mut self) {
        let mut to_decode = self.data.iter().cloned().rev().collect::<Vec<u8>>();
        let mut res = Vec::with_capacity(to_decode.len() * 3);

        while !to_decode.is_empty() {
            let freq_upper = to_decode.pop().unwrap();
            let freq_lower = to_decode.pop().unwrap();
            let freq = (freq_upper as u16) << 8 | freq_lower as u16;
            let symbol = to_decode.pop().unwrap();
            res.extend(vec![symbol; freq as usize])
        }

        self.data = res;
    }
}