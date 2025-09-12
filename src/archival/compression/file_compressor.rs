use std::path::PathBuf;

pub struct Compressor {
    pub data: Vec<u8>,
    pub method: u8,
    // pub ratio: f32,
    // pub start: Instant,
}

impl Compressor {
    pub fn new(data: Vec<u8>, method: u8) -> Self {
        Self {
            data,
            method,
        }
    }

    pub fn compress(&mut self) -> Vec<u8>{
        let (
            rle,
            rle2,
            _,
            _,
            _,
            _,
            _,
            _
        ) = (
            self.method & (1 << 7) != 0,
            self.method & (1 << 6) != 0,
            self.method & (1 << 5) != 0,
            self.method & (1 << 4) != 0,
            self.method & (1 << 3) != 0,
            self.method & (1 << 2) != 0,
            self.method & (1 << 1) != 0,
            self.method & (1 << 0) != 0,
        );

        if rle && !rle2 { self.rle(); }
        if rle2 { self.rle_two_byte(); }

        self.data.clone()
    }

    pub fn decompress(&mut self) -> Vec<u8> {
        let (
            rle,
            rle2,
            _,
            _,
            _,
            _,
            _,
            _
        ) = (
            self.method & (1 << 7) != 0,
            self.method & (1 << 6) != 0,
            self.method & (1 << 5) != 0,
            self.method & (1 << 4) != 0,
            self.method & (1 << 3) != 0,
            self.method & (1 << 2) != 0,
            self.method & (1 << 1) != 0,
            self.method & (1 << 0) != 0,
        );

        if rle && !rle2 { self.decompress_rle(); }
        if rle2 { self.decompress_rle_two_byte(); }

        self.data.clone()
    }
}