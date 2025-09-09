use crate::archival::cli::input::{Command, Mode};
use crate::archival::cli::output::FmtProgress;
use crate::archival::compression::file_compressor::Compressor;
use crate::archival::compression::profiler::Profiler;
use crate::archival::files::header::{ArchiveHeader, FileHeader, Header};
use crate::archival::files::indexer::{ArchiveIndexer, FileRange};
use crate::constants::{GIGABYTE, KILOBYTE, MEGABYTE, TERABYTE};
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Read, Write};
use std::path::PathBuf;
use std::time::Instant;
use std::fs;

macro_rules! format_bytes {
    ($s: expr) => {
        // not hacky in the slightest
        match $s as u64  {
            0..KILOBYTE => format!("{} bytes", $s),
            KILOBYTE..MEGABYTE => format!("{:.1} KB", $s as f64 / KILOBYTE as f64),
            MEGABYTE..GIGABYTE => format!("{:.1} MB", $s as f64 / MEGABYTE as f64),
            GIGABYTE..TERABYTE => format!("{:.1} GB", $s as f64 / GIGABYTE as f64),
            _ => format!("{:.1} TB", $s as f64 / TERABYTE as f64),
        }
    };
}

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
    pub archive_size: u64,
    pub speed: usize,

    pub ranges: Vec<FileRange>,
    pub buffer_size: usize,

    pub archive_reader: Option<BufReader<File>>,
}

pub struct ArchivalError(pub String);

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
            None => { todo!("return error that no output was given") }
        };


        let mut index = ArchiveIndexer::new(input.clone());
        index.index_files();
        let files = index.contents;
        let file_count = index.file_count;
        let total_bytes = index.bytes_count;
        let ranges = index.ranges;

        // if archive is being extracted, get the total archive size
        let archive_size = match mode {
            Mode::Extract => {
                input.metadata().unwrap().len()
            }
            _ => 0
        };

        // if archive is being extracted, create a buffer for reading it
        let archive_reader = match mode {
            Mode::Extract => {
                Some(BufReader::new(File::open(&input).unwrap()))
            },
            _ => None
        };

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
            archive_size,
            speed: 0,
            ranges,
            buffer_size: 0,
            archive_reader,
        }
    }

    /// reads an archive header and returns its data
    pub fn read_archive_header(&mut self) -> Result<ArchiveHeader, ArchivalError> {
        // FIXME : this code is ugly as fuck
        //   & must be cleaned up for the love of all that is holy
        let mut lines = Vec::<String>::with_capacity(Header::ARCHIVE_HEADER_SIZE);

        for _ in 0..Header::ARCHIVE_HEADER_SIZE {
            let mut line = String::new();
            let bytes_read = self.archive_reader
                .as_mut()
                .unwrap()
                .read_line(&mut line);
            if bytes_read.is_err() {
                return Err(ArchivalError(
                    String::from(format!(
                        "failed to read archive header: {}",
                        bytes_read.err().unwrap()
                    ))
                ))
            }
            self.bytes_processed += bytes_read.unwrap();
            lines.push(line.trim().to_string());
        }

        let data = lines.iter().map(|l| {
            l.split(':').collect::<Vec<&str>>()
        }).collect::<Vec<Vec<&str>>>();
        let total_files = data[0][1].parse::<usize>().unwrap();
        let version = data[1][1].parse::<usize>().unwrap();
        let encrypted = data[2][1].parse::<bool>().unwrap();

        Ok(ArchiveHeader(total_files, version, encrypted))
    }

    /// reads a file header and returns its data
    pub fn read_file_header(&mut self) -> Result<FileHeader, ArchivalError> {
        // read header lines
        let mut lines = Vec::with_capacity(Header::FILE_HEADER_SIZE);
        for _ in 0..Header::FILE_HEADER_SIZE {
            let mut line = String::new();
            let bytes_read = self.archive_reader
                .as_mut()
                .unwrap()
                .read_line(&mut line);
            if bytes_read.is_err() {
                return Err(ArchivalError(
                    String::from(format!(
                        "failed to read file header: {}\n\
                        current line contents: \"{}\"\n\
                        current line bytes: \"{:?}\"\n\
                        lines read: {:?}",
                        bytes_read.err().unwrap(), line, line.as_bytes(), lines
                    ))
                ))
            }
            self.bytes_processed += bytes_read.unwrap();
            lines.push(line.trim().to_string());
        }

        // parse lines
        let data = lines.iter().map(|l| {
            l.split(':').collect::<Vec<&str>>()
        }).collect::<Vec<Vec<&str>>>();
        let name = data[0][1].parse::<String>().unwrap();
        let method = data[1][1].parse::<u8>().unwrap();
        let compressed = data[2][1].parse::<u64>().unwrap();
        let decompressed = data[3][1].parse::<u64>().unwrap();

        // return header data
        Ok(FileHeader(name, method, compressed, decompressed))
    }

    /// Compile the files from the input path into the output archive.
    /// if the archive does exist it will be updated with the given files,
    /// otherwise it will simply be created from
    /// all files contained in the input path
    pub fn add(&mut self) -> Result<u64, ArchivalError>{
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

        // create and write the archive header
        let header = Header::Archive {
            total_files: self.file_count,
            // todo : update this value if the archive is being updated,
            //   otherwise it is zero when the archive is created
            version: 0,
            encrypted: false,
        };
        let write_res = buffer.write(header.to_bytes().as_slice());
        if write_res.is_err() {
            return Err(ArchivalError(
                format!(
                    "Failed to write archive header: {}",
                    write_res.err().unwrap())
            ))
        }

        // write data from all files into one
        for (i, path) in self.files.iter().enumerate() {
            let input_file = File::open(path.clone());
            if input_file.is_err() {
                return Err(ArchivalError(
                    format!(
                        "Could not open input file: \"{}\"\nreason: {}",
                        path.display(),
                        input_file.err().unwrap())
                ))
            }
            let input_file = input_file.unwrap();
            let metadata = input_file.metadata().unwrap();

            // profile the file to determine the best method to compress it
            let mut file_profile = Profiler::new(path.clone());
            file_profile.profile();
            let method = file_profile.to_method();

            // compress the file data
            let mut file_compressor = Compressor::new(
                fs::read(path).unwrap(), method
            );
            let new_data = file_compressor.compress();

            // create and write file header
            let relative_path = path.strip_prefix(&self.input)
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let header = Header::File {
                name: relative_path,
                method,
                compressed_size: new_data.len() as u64,
                decompressed_size: metadata.len(),
            };

            let write_res = buffer.write(header.to_bytes().as_slice());
            if write_res.is_err() {
                return Err(ArchivalError(
                    format!(
                        "Could not write to output file: {}",
                        write_res.err().unwrap())
                ))
            }

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

            buffer.write_all(new_data.as_slice()).unwrap();

            // logging
            self.files_processed += 1;
            self.bytes_processed += new_data.len();
            self.speed = (self.bytes_processed as f64 /
                self.start_time.unwrap().elapsed().as_secs_f64()
            ) as usize;

            self.format_progress(format!("{}", path.display()));
        }

        let speed = format_bytes!(self.speed);

        println!(
            "Archival Completed in {:.2}s with a speed of {} per second",
            self.start_time.unwrap().elapsed().as_secs_f64(),
            speed,
        );

        Ok(self.archive_size)
    }

    /// Extract the contents of an archive into the output path
    pub fn extract(&mut self) -> Result<u64, ArchivalError> {
        let ArchiveHeader(files, _ver, encrypted) = self.read_archive_header()?;

        for i in 0..files {
            // read header
            let FileHeader(name, method, compressed, decompressed) = self.read_file_header()?;

            // read 'n' bytes specified by the header
            // we reserve an extra byte to account for the
            // leading newline into the next file header
            let mut buffer = if i == files - 1 {
                vec![0; compressed as usize]
            } else {
                vec![0; compressed as usize + 1]
            };

            let read_res = self.archive_reader.as_mut().unwrap().read_exact(&mut buffer);
            if let Err(e) = read_res {
                return Err(ArchivalError(
                    format!("failed to read file data: {}", e)
                ))
            }

            let buffer = if i == files - 1 {
                buffer.as_ref()
            } else {
                &buffer[0..compressed as usize]
            };

            let mut decompressor = Compressor::new(buffer.to_vec(), method);
            let decompressed_data = decompressor.decompress();

            // reconstruct the files into a dir with the same name as the archive
            let path = self.output.clone().join(PathBuf::from(&name));
            fs::create_dir_all(&path.parent().unwrap()).unwrap();
            fs::write(&path, decompressed_data).unwrap();
            self.format_progress(format!("{}", path.display()));

            // logging
            self.bytes_processed += buffer.len();
            self.files_processed += 1;
            self.speed = (self.bytes_processed as f64 /
                self.start_time.unwrap().elapsed().as_secs_f64()
            ) as usize;
        }

        let speed = format_bytes!(self.speed);

        println!(
            "Extraction Completed in {:.2}s with a speed of {} per second",
            self.start_time.unwrap().elapsed().as_secs_f64(),
            speed,
        );

        Ok(self.archive_size)
    }

    /// profile the input path and log the details
    pub fn profile(&mut self) -> Result<u64, ArchivalError> {
        if self.input.is_dir() {
            return Err(ArchivalError(
                "Profiling target must be a file!".to_owned()
            ))
        }
        let mut profiler = Profiler::new(self.input.clone());
        profiler.profile();
        println!("Profile: \n{:?}", profiler);
        Ok(0)
    }

    /// general function to run all operations specified by the command
    // todo : maybe a better name ig
    pub fn operate(&mut self) -> Result<u64, ArchivalError> {
        self.start_time = Some(Instant::now());
        match self.mode {
            Mode::Add => {
                self.add()
            },
            Mode::Extract => {
                self.extract()
            }
            Mode::Profile => {
                self.profile()
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

