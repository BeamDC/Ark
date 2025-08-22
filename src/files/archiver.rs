use std::path::PathBuf;
use crate::cli::input::{Command, Mode};
use crate::files::indexer::ArchiveIndexer;

pub struct Archiver {
    pub mode: Mode,

    pub input: PathBuf,
    pub output: PathBuf,
    pub files: Vec<PathBuf>,
    pub file_count: usize,
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

        Archiver {
            mode,
            input,
            output,
            files,
            file_count,
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
        todo!()
    }

    /// general function to run all operations specified by the command
    // todo : maybe a better name ig
    pub fn operate(&mut self) {
        match self.mode {
            Mode::Add => {
                todo!("add files to archive")
            },
            Mode::Extract => {
                todo!("extract the archive")
            }
        }
    }
}

