use std::io::{stdout, Write};
use std::path::PathBuf;
use std::time::Instant;
use walkdir::WalkDir;


pub struct ArchiveIndexer {
    pub root: PathBuf,
    pub contents: Vec<PathBuf>,
    pub read_start_time: Option<Instant>,
    pub file_count: usize,
    pub bytes_count: usize,
}

impl ArchiveIndexer {
    pub fn new(root: PathBuf) -> ArchiveIndexer {
        ArchiveIndexer {
            root,
            contents: vec![],
            read_start_time: None,
            file_count: 0,
            bytes_count: 0,
        }
    }

    /// recursively read all file paths contained within the root directory.
    /// Pushes all file paths to `self.contents`
    /// and tracks the total number of files with `self.file_count`
    pub fn index_files(&mut self) {
        self.read_start_time = Some(Instant::now());

        let mut contents = WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| entry.path().to_path_buf())
            .collect::<Vec<PathBuf>>();

        // Sort by file size, smallest first
        contents.sort_by_key(|path| {
            std::fs::metadata(path)
                .map(|m| m.len())
                .unwrap_or(u64::MAX) // Put files we can't read at the end
        });

        self.bytes_count = WalkDir::new(&self.root)
            .into_iter()
            .map(|item| item.unwrap().metadata().unwrap().len() as usize)
            .sum();

        self.file_count = contents.len();
        self.contents = contents;

        for entry in self.contents.clone() {
            println!("indexed :: {}", entry.to_str().unwrap());
            // stdout().flush().unwrap();
        }

        println!(
            "Successfully indexed {} files in {:.2}s",
            self.file_count,
            self.read_start_time.unwrap().elapsed().as_secs_f64()
        );
    }
}