use std::path::PathBuf;
use crate::organization::data::Tag;

#[derive(Clone, Debug)]
pub struct VidItem {
    pub path: PathBuf,
    pub name: String,
    pub extension: String,
    pub tags: Vec<Tag>,
    pub selected: bool,
}

impl VidItem {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path: path.clone(),
            name: path.file_stem().unwrap().to_str().unwrap().to_string(),
            extension: path.extension().unwrap().to_str().unwrap().to_string(),
            ..Self::default()
        }
    }

    pub fn path_as_string(&self) -> String {
        self.path.display().to_string()
    }
}

impl Default for VidItem {
    fn default() -> Self {
        Self {
            path: PathBuf::new(),
            name: String::new(),
            extension: String::new(),
            tags: vec![],
            selected: false,
        }
    }
}

impl PartialEq for VidItem {
    fn eq(&self, other: &VidItem) -> bool {
        self.path == other.path
            && self.name == other.name
            && self.extension == other.extension
            && self.tags == other.tags
    }
}