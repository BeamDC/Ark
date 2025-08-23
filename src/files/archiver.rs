use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{copy, stdout, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;
use crate::cli::input::{Command, Mode};
use crate::cli::output::FmtProgress;
use crate::files::indexer::ArchiveIndexer;

pub struct Archiver {
    pub mode: Mode,

    pub input: PathBuf,
    pub output: PathBuf,
    pub files: Vec<PathBuf>,
    pub file_count: usize,
    pub total_bytes: usize,

    pub start_time: Option<Instant>,
    pub bytes_processed: usize,
    pub files_processed: usize,
    pub speed: usize,
}

impl Archiver {
    /// construct a new `Archiver` from a `Command`
    pub fn new(command: Command) -> Archiver {
        let mode = match command.mode {
            Some(mode) => mode,
            None => todo!("return error that no mode was given")
        };

        let input = match command.input {
            Some(input) => input,
            None => {todo!("return error that no input was given")}
        };

        let output = match command.output {
            Some(output) => output,
            None => {todo!("return error that no output was given")}
        };


        let mut index = ArchiveIndexer::new(input.clone());
        index.index_files();
        let files = index.contents;
        let file_count = index.file_count;
        let total_bytes = index.bytes_count;

        Archiver {
            mode,
            input,
            output,
            files,
            file_count,
            total_bytes,
            start_time: None,
            bytes_processed: 0,
            files_processed: 0,
            speed: 0,
        }
    }

    /// Extract the contents of an archive into the output path
    pub fn extract(&mut self) {
        todo!()
    }

    /// Compile the files from the input path into the output archive.
    /// if the archive does exist it will be updated with the given files,
    /// otherwise it will simply be created from
    /// all files contained in the input path
    pub fn add(&mut self) {
        let buffer_size = 1024 * 1024 * 32;
        let output_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output)
            .unwrap_or_else(|_| {
                todo!("return error when file cannot be opened")
            });

        let mut buffer = BufWriter::with_capacity(buffer_size, output_file);

        // todo : write header

        // write data from all files into one
        for file in &self.files {
            let input_file = File::open(file).unwrap_or_else(|_| {
                todo!("return error that file open failed")
            });
            let mut reader = BufReader::with_capacity(buffer_size, input_file);

            // Stream copy instead of loading everything into memory
            let bytes_copied = copy(&mut reader, &mut buffer).unwrap_or_else(|_| {
                todo!("return error that copy failed")
            });

            self.files_processed += 1;
            self.bytes_processed += bytes_copied as usize;

            // self.format_progress(format!("{}", file.display()));
            // stdout().flush().unwrap();
        }
        self.speed = (self.total_bytes as f64 /
            self.start_time.unwrap().elapsed().as_secs_f64()
        ) as usize;

        // not hacky in the slightest
        let speed = match self.speed  {
            0..=1023 => format!("{} bytes", self.speed),
            1024..=1048575 => format!("{:.1} KB", self.speed as f64 / 1024.0),
            1048576..=1073741823 => format!("{:.1} MB", self.speed as f64 / (1024.0 * 1024.0)),
            1073741824..=1099511627775 => format!("{:.1} GB", self.speed as f64 / (1024.0 * 1024.0 * 1024.0)),
            _ => format!("{:.1} TB", self.speed as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)),
        };

        println!("Archival Completed in {:.2}s with a speed of {} per second",
                 self.start_time.unwrap().elapsed().as_secs_f64(),
                 speed,
        );
    }

    /// general function to run all operations specified by the command
    // todo : maybe a better name ig
    pub fn operate(&mut self) {
        self.start_time = Some(Instant::now());
        match self.mode {
            Mode::Add => {
                self.add()
            },
            Mode::Extract => {
                todo!("extract the archive")
            }
        }
    }
}

impl FmtProgress for Archiver {
    fn get_progress_percentage(&self) -> Option<f64> {
        Some((self.bytes_processed as f64 / self.total_bytes as f64) * 100.0)
    }

    fn get_estimated_time_remaining(&self) -> Option<f64> {
        let remaining_bytes = self.total_bytes.saturating_sub(self.bytes_processed) as f64;
        Some(remaining_bytes / self.speed as f64)
    }
}

