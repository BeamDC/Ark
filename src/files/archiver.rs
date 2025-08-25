use crate::cli::input::{Command, Mode};
use crate::cli::output::FmtProgress;
use crate::files::indexer::{ArchiveIndexer, FileRange};
use std::fs::{File, OpenOptions};
use std::io::{copy, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::time::Instant;
use std::cmp;

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
    pub archive_size: usize,
    pub speed: usize,

    pub ranges: Vec<FileRange>,
    pub buffer_size: usize,
}

pub struct ArchivalError(String);

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
        let ranges = index.ranges;

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
            archive_size: 0,
            speed: 0,
            ranges,
            buffer_size: 0,
        }
    }

    /// Compile the files from the input path into the output archive.
    /// if the archive does exist it will be updated with the given files,
    /// otherwise it will simply be created from
    /// all files contained in the input path
    pub fn add(&mut self) -> Result<usize, ArchivalError>{
        let output_file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.output);
        if output_file.is_err() {
            return Err(
                ArchivalError(String::from("Could not open output file"))
            )
        }
        let output_file = output_file.unwrap();

        let mut buffer = BufWriter::with_capacity(self.buffer_size, output_file);

        // todo : write archive header

        // write data from all files into one
        for (i, file) in self.files.iter().enumerate() {
            // todo : file header

            // get current file range
            let current_range = self.ranges.get(
                self.ranges.iter().position(|fr| {
                    i >= fr.range.0 && i < fr.range.1
                }).unwrap()
            );
            if current_range.is_none() {
                return Err(ArchivalError(
                    format!("Could not find range for file indexed at {}", i)
                ))
            }
            let current_range = current_range.unwrap();

            // resize buffer
            if current_range.buffer_size > self.buffer_size {
                self.buffer_size = current_range.buffer_size;

                let flush = buffer.flush();
                if flush.is_err() {
                    return Err(ArchivalError(
                        String::from("failed to flush buffer during resize")
                    ))
                }
                let writer = buffer.into_inner();
                if writer.is_err() {
                    return Err(ArchivalError(
                        String::from("failed to get inner writer from buffer")
                    ))
                }
                let writer = writer.unwrap();

                buffer = BufWriter::with_capacity(self.buffer_size, writer);
            }

            // read file
            let input_file = File::open(file);
            if input_file.is_err() {
                return Err(ArchivalError(
                    String::from("Could not open input file")
                ))
            }
            let input_file = input_file.unwrap();
            let reader_size = cmp::min(
                self.buffer_size,
                input_file.metadata().unwrap().len() as usize
            );
            let mut reader = BufReader::with_capacity(
                reader_size,
                input_file
            );

            // stream copy instead of loading everything into memory
            let bytes_copied = copy(&mut reader, &mut buffer);
            if bytes_copied.is_err() {
                return Err(ArchivalError(
                    String::from("failed to copy to buffer")
                ))
            }
            let bytes_copied = bytes_copied.unwrap();

            // logging
            self.files_processed += 1;
            self.bytes_processed += bytes_copied as usize;

            self.format_progress(format!("{}", file.display()));

            self.speed = (self.bytes_processed as f64 /
                self.start_time.unwrap().elapsed().as_secs_f64()
            ) as usize;
        }

        // not hacky in the slightest
        let speed = match self.speed  {
            0..=1023 => format!("{} bytes", self.speed),
            1024..=1048575 => format!("{:.1} KB", self.speed as f64 / 1024.0),
            1048576..=1073741823 => format!("{:.1} MB", self.speed as f64 / (1024.0 * 1024.0)),
            1073741824..=1099511627775 => format!("{:.1} GB", self.speed as f64 / (1024.0 * 1024.0 * 1024.0)),
            _ => format!("{:.1} TB", self.speed as f64 / (1024.0 * 1024.0 * 1024.0 * 1024.0)),
        };

        println!(
            "Archival Completed in {:.2}s with a speed of {} per second",
            self.start_time.unwrap().elapsed().as_secs_f64(),
            speed,
        );

        Ok(self.archive_size)
    }

    /// Extract the contents of an archive into the output path
    pub fn extract(&mut self) {
        todo!()
    }

    /// general function to run all operations specified by the command
    // todo : maybe a better name ig
    pub fn operate(&mut self) -> Result<usize, ArchivalError> {
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
        if self.speed == 0 {
            return None
        }
        Some(remaining_bytes / self.speed as f64)
    }

    fn get_current_speed(&self) -> Option<usize> {
        Some(self.speed)
    }
}

