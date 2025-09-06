use std::path::PathBuf;
use crate::organization::data::Tag;
use crate::organization::item::VidItem;
use crate::organization::log::LogData;

#[test]
fn test_read() {
    let data = LogData::read_log(PathBuf::from("src/tests"));

    assert_eq!(data.root, PathBuf::from("path/to/root/dir/"));
    assert_eq!(data.file_count, 2);
    assert_eq!(data.tags, vec![
        Tag::new("a".to_owned()),
        Tag::new("b".to_owned()),
        Tag::new("c".to_owned()),
        Tag::new("d".to_owned()),
    ]);
    assert_eq!(data.items[0], VidItem {
        path: PathBuf::from("file1.mov"),
        name: "file1".to_string(),
        extension: "mov".to_string(),
        tags: vec![Tag::new("a".to_owned()), Tag::new("b".to_owned())],
        selected: false,
    });
    assert_eq!(data.items[1], VidItem {
        path: PathBuf::from("file2.mp4"),
        name: "file2".to_string(),
        extension: "mp4".to_string(),
        tags: vec![Tag::new("c".to_owned()), Tag::new("d".to_owned())],
        selected: false,
    });
}