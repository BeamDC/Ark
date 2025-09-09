use std::fs;
use std::fs::File;
use std::path::PathBuf;
use crate::organization::data::Tag;
use crate::organization::item::VidItem;

#[derive(Debug)]
pub struct LogData {
    pub root: PathBuf,
    pub file_count: u64,
    pub tags: Vec<Tag>,
    pub items: Vec<VidItem>
}

impl LogData {
    pub fn new(root: PathBuf, file_count: u64, tags: Vec<Tag>, items: Vec<VidItem>) -> Self {
        Self { root, file_count, tags, items }
    }

    /// push all changes to `ark.data` saving all files and tags.
    ///
    /// This will also bring all changed files not yet in root, to the root
    ///
    /// `ark.data` is formatted in the following way, with an initial header
    /// containing important information, followed by all relevant files
    /// and their unique information.
    ///
    /// # Example
    /// ```ignore
    /// path/to/root/dir/..   // root dir
    /// 512                   // total file count
    /// a,b,c,d, etc.         // all known tags
    ///
    /// // file format is the following:
    /// // name.ext ; tag list
    /// file1.mp4;a,b
    /// file2.mov;a,d,e
    /// ...
    /// ```
    pub fn update_save(&self) {
        let data_path = self.root.join("ark.data");
        if !data_path.exists() {
            File::create(&data_path).unwrap();
        }
        todo!("gather all important data and write it to the file")
    }

    /// write all file data to `ark.data`
    pub fn write_log(&self, data: LogData) {
        todo!("write header");
        todo!("write body");
    }

    /// check if a file already exists and is contained in the current log
    fn check_entry(&self, entry: PathBuf) -> bool {
        for item in &self.items {
            if item.path == entry {
                return true
            }
        }
        false
    }

    /// read and parse `ark.data` into `LogData`
    ///
    /// # Example
    /// ```ignore
    ///     /// the following are the file contents being parsed
    ///     // path/to/root/dir/
    ///     // 2
    ///     // a,b,c,d
    ///     // file1.mov;a,b
    ///     // file2.mp4;c,d
    ///
    ///
    ///    let data = LogData::read_log(PathBuf::from("src/tests"));
    ///
    ///    assert_eq!(data.root, PathBuf::from("path/to/root/dir/"));
    ///    assert_eq!(data.file_count, 2);
    ///    assert_eq!(data.tags, vec![
    ///        Tag::new("a".to_owned()),
    ///        Tag::new("b".to_owned()),
    ///        Tag::new("c".to_owned()),
    ///        Tag::new("d".to_owned()),
    ///    ]);
    ///    assert_eq!(data.items[0], VidItem {
    ///        path: PathBuf::from("file1.mov"),
    ///        name: "file1".to_string(),
    ///        extension: "mov".to_string(),
    ///        tags: vec![Tag::new("a".to_owned()), Tag::new("b".to_owned())],
    ///        selected: false,
    ///    });
    /// ```
    pub fn read_log(path: PathBuf) -> LogData {
        let data_path = path.join("ark.data");
        if !data_path.exists() {
            return LogData::default();
        }

        // treat data like a stack since its easy to use :P
        let data = fs::read_to_string(&data_path).unwrap();
        let mut data = data
            .split('\n')
            .rev()
            .collect::<Vec<&str>>();

        // pull raw data
        let root = data.pop().expect("Header Error: no root found");
        let file_count = data.pop().expect("Header Error: no file count found");
        let all_tags = data.pop().expect("Header Error: no tags found");

        // parse data
        let root = PathBuf::from(
            // windows ends lines with a carriage return as well as a newline
            // we want to get rid of the carriage return if it exists
            root.strip_suffix("\r").unwrap_or(root)
        );

        let file_count = file_count
            .strip_suffix("\r")
            .unwrap_or(file_count)
            .parse::<u64>()
            .expect("Header Error");

        let all_tags = all_tags
            .strip_suffix("\r")
            .unwrap_or(all_tags)
            .split(',')
            .map(|t| { Tag::new(t.to_owned()) })
            .collect::<Vec<Tag>>();

        let mut items = vec![];
        while !data.is_empty() {
            let current = data.pop().expect("Content Error: no entry found");
            let (name_with_ext, tags) = {
                let mut current = current
                    .strip_suffix('\r')
                    .unwrap_or(current)
                    .split(';');

                (
                    current.next().expect("Missing file name and extension"),
                    current.next().expect("Missing tags"),
                )
            };

            let (name, extension) = (
                PathBuf::from(name_with_ext).file_stem()
                    .unwrap().to_str().unwrap().to_owned(),
                PathBuf::from(name_with_ext).extension()
                    .unwrap().to_str().unwrap().to_owned(),
            );

            let tags = tags
                .strip_suffix("\r")
                .unwrap_or(tags)
                .split(',')
                .map(|t| { Tag::new(t.to_owned()) })
                .collect::<Vec<Tag>>();

            items.push(VidItem {
                path: PathBuf::from(name_with_ext),
                name,
                extension,
                tags,
                ..Default::default()
            });
        }

        LogData {
            root,
            file_count,
            tags: all_tags,
            items
        }
    }
}

impl Default for LogData {
    fn default() -> Self {
        LogData {
            root: PathBuf::new(),
            file_count: 0,
            tags: vec![],
            items: vec![]
        }
    }
}