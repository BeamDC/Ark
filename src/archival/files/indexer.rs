use std::path::PathBuf;
use std::time::Instant;
use walkdir::WalkDir;

#[derive(Clone)]
pub struct FileRange {
    pub range: (usize, usize),
    pub buffer_size: usize,
}

pub struct ArchiveIndexer {
    pub root: PathBuf,
    pub contents: Vec<PathBuf>,
    pub read_start_time: Option<Instant>,
    pub file_count: usize,
    pub bytes_count: usize,
    pub ranges: Vec<FileRange>,
}

impl ArchiveIndexer {
    fn get_ideal_buffer_size(bytes: u64) -> usize {
        match bytes {
            // 0b - 100mb -> 256kb
            0..104_857_600 => {
                1024 * 256
            }
            // 100mb - 500mb -> 1mb
            104_857_600..524_288_005 => {
                1024 * 1024 * 1
            }
            // > 500mb -> 8mb
            _ => {
                1024 * 1024 * 8
            }
        }
    }

    pub fn new(root: PathBuf) -> ArchiveIndexer {
        ArchiveIndexer {
            root,
            contents: vec![],
            read_start_time: None,
            file_count: 0,
            bytes_count: 0,
            ranges: vec![],
        }
    }

    /// recursively read all file paths contained within the root directory.
    /// Pushes all file paths to `self.contents`
    /// and tracks the total number of files with `self.file_count`
    pub fn index_files(&mut self) {
        self.read_start_time = Some(Instant::now());

        let contents = WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.file_type().is_file())
            .map(|entry| {
                println!("indexed :: {}", entry.path().to_str().unwrap());
                entry.path().to_path_buf()
            })
            .collect::<Vec<PathBuf>>();

        // Sort by file size, smallest first
        // contents.sort_by_key(|path| {
        //     std::fs::metadata(path)
        //         .map(|m| m.len())
        //         .unwrap_or(u64::MAX) // Put files we can't read at the end
        // });

        // get files sizes
        let sizes = WalkDir::new(&self.root)
            .into_iter()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.metadata().unwrap().len())
            .collect::<Vec<u64>>();

        // find file ranges
        let (mut start, mut end) = (0, 0);
        let mut prev_buffer_size = None;
        let mut current_buffer_size;
        for size in sizes {
            current_buffer_size = Some(Self::get_ideal_buffer_size(size));

            // when a major jump in size occurs,
            // we push the current range and recalculate
            if  prev_buffer_size.is_some() &&
                prev_buffer_size != current_buffer_size {
                self.ranges.push(FileRange {
                    range: (start, end),
                    buffer_size: prev_buffer_size.unwrap(),
                });
                start = end;
            }

            prev_buffer_size = current_buffer_size;
            end += 1;
        }
        // push last range if needed
        if start != end - 1 {
            self.ranges.push(FileRange {
                range: (start, end),
                buffer_size: prev_buffer_size.unwrap(),
            });
        }

        // get total size
        self.bytes_count = WalkDir::new(&self.root)
            .into_iter()
            .map(|item| item.unwrap().metadata().unwrap().len() as usize)
            .sum();

        self.file_count = contents.len();
        self.contents = contents;

        println!(
            "Successfully indexed {} files in {:.2}s",
            self.file_count,
            self.read_start_time.unwrap().elapsed().as_secs_f64()
        );
    }
}