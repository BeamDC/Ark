use std::{fmt, fs};
use std::cmp::min;
use std::ffi::OsStr;
use std::fmt::Formatter;
use std::path::PathBuf;
use std::time::Instant;
use crate::constants::{GIGABYTE, MEGABYTE};

/// a profiler for determining useful methods for compressing a file
pub struct Profiler {
    pub file: PathBuf,
    pub data: Vec<u8>,
    pub start: Option<Instant>,

    // RLE data
    pub rle: bool,          // will be true if rle is recommended
    pub two_byte_rle: bool, // will be true if two byte rle is recommended
    pub avg_run_len: f32,   // the average run length of the bytes in this file
}

impl Default for Profiler {
    fn default() -> Self {
        Self {
            file: PathBuf::new(),
            data: vec![],
            start: None,
            rle: false,
            two_byte_rle: false,
            avg_run_len: 0.0,
        }
    }
}

impl fmt::Debug for Profiler {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", format!("Profile for {}:", self.file.display()))?;
        writeln!(f, "{}", format!("  - RLE recommended -> {}", self.rle))?;
        writeln!(f, "{}", format!("  - two byte RLE recommended -> {}", self.two_byte_rle))?;
        writeln!(f, "{}", format!("  - AVG run length  -> {}", self.avg_run_len))?;
        writeln!(f, "{}", format!("Profiling completed in {}ms", self.start.unwrap().elapsed().as_millis()))
    }
}

impl Profiler {
    pub fn new(file: PathBuf) -> Self {
        let data = fs::read(&file).unwrap();
        Self {
            file,
            data,
            ..Default::default()
        }
    }

    /// Checks the file data to determine whether
    /// run length encoding is worth using.
    ///
    /// Run length encoding is beneficial when a file consists
    /// mostly of runs of a length greater than two bytes.
    ///
    /// This is because run length encoding ensures that any run
    /// -- when encoded -- has a length of two bytes,
    /// as such we have 3 cases:
    ///
    /// 1. If a run contains a single byte, it is resized to two bytes.
    ///
    /// 2. If a run contains exactly two bytes, its size does not change.
    ///
    /// 3. If a run contains three or more bytes (up to 255),
    /// it is resized to two and is thus smaller than its original size.
    ///
    /// as such, if the majority of runs in a file are of the case where
    /// they have a length of three of greater, then run length encoding
    /// will result in a decrease of total file size.
    ///
    /// for this implementation we add an extra check for when
    /// average run length is greater than 255. this is because for
    /// bitmap images and other data where rle may be effective,
    /// it is possible, or even likely to encounter runs with lengths
    /// that far exceed 255, at this point it is worth investing an extra byte
    /// of run length in order to compress these long runs further
    fn validate_rle(&mut self) {
        let ext = self.file.extension()
            .unwrap_or_else(|| OsStr::new(""))
            .to_ascii_lowercase();
        match ext.to_str().unwrap() {
            "bmp" | "tiff" | "pdf" => {}
            _ => return,
        }

        self.avg_run_len = if self.data.len() > MEGABYTE as usize {
            let sample_size: usize = min(2048, self.data.len() / 4);
            let mut start = self.data[..sample_size].to_vec();
            let mid = self.data.len() / 2;
            let mut mid = self.data[mid..sample_size + mid].to_vec();
            let mut end = self.data[self.data.len() - sample_size..].to_vec();

            let size = start.len() + mid.len() + end.len();

            start.dedup();
            mid.dedup();
            end.dedup();

            let runs = start.len() + mid.len() + end.len();

            size as f32 / runs.max(1) as f32
        } else {
            let mut current = None;
            let mut runs = 0u64;
            for byte in &self.data {
                if current.is_none() || byte != current.unwrap() {
                    runs += 1;
                    current = Some(byte);
                }
            }

            self.data.len() as f32 / runs.max(1) as f32
        };


        self.rle = match &self.avg_run_len {
            // not recommended
            0.0..=2.0 => false,
            // recommended
            2.0..=255.0 => true,
            // two byte recommended
            _ => {
                self.two_byte_rle = true;
                true
            },
        }
    }

    #[must_use]
    pub fn profile(&mut self) -> u8 {
        self.start = Some(Instant::now());

        // some files may be too big to be worth compressing
        if self.file.metadata().unwrap().len() >= GIGABYTE {
            return 0
        }

        self.validate_rle();
        self.to_method()
    }

    /// returns a u8 where bits are mapped to different algorithms used to compress the data
    /// the mapping is such that the largest bit (assuming little endian)
    /// represents the first operation performed
    ///
    /// 1 -> is rle used?
    ///
    /// 1 -> is 2-bit rle used?
    ///
    /// 0 ->
    ///
    /// 0 ->
    ///
    /// 0 ->
    ///
    /// 0 ->
    ///
    /// 0 ->
    ///
    /// 1 ->
    ///
    /// in binary the above method is represented as: `11000001`
    pub fn to_method(&self) -> u8 {
        let method = 0u8
            | ((self.rle as u8) << 7)
            | ((self.two_byte_rle as u8) << 6)
            // ... put other method stuff here ...
            ;
        method
    }
}



