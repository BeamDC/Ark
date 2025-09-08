use std::path::PathBuf;
use crate::archival::cli::input::{Command, Mode};
use crate::archival::files::archiver::Archiver;

#[test]
fn test_read_archive() {
    let mut archiver = Archiver::new(Command {
        mode: Some(Mode::Extract),
        input: Some(PathBuf::from("src/tests/archive.ark")),
        // filler output
        output: Some(PathBuf::from("src/tests/archive.ark")),
        key: None,
    });

    if let Ok(head) = archiver.read_archive_header() {
        assert_eq!(2, head.0);
        assert_eq!(0, head.1);
        assert_eq!(false, head.2);
    } else { panic!("Archive header failed to be read!") }

    if let Ok(head) = archiver.read_file_header() {
        assert_eq!("file1.mov".to_owned(), head.0);
        assert_eq!(0u8, head.1);
        assert_eq!(50, head.2);
        assert_eq!(60, head.3);
    }
}