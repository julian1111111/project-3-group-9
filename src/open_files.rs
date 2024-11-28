use std::collections::HashMap;

#[derive(Clone, PartialEq)]
pub enum FileMode {
    ReadOnly,
    WriteOnly,
    ReadWrite,
}

pub struct OpenFile {
    pub filename: String,
    pub mode: FileMode,
    pub offset: u32,
    pub first_cluster: u32,
    pub file_size: u32,
}

pub struct OpenFiles {
    files: HashMap<String, OpenFile>,
}

impl OpenFiles {
    pub fn new() -> Self {
        OpenFiles {
            files: HashMap::new(),
        }
    }

    pub fn open_file(&mut self, file: OpenFile) -> Result<(), String> {
        if self.files.len() >= 10 {
            return Err("Error: Maximum number of open files reached.".to_string());
        }
        if self.files.contains_key(&file.filename) {
            return Err(format!("Error: File '{}' is already open.", file.filename));
        }
        self.files.insert(file.filename.clone(), file);
        Ok(())
    }

    pub fn close_file(&mut self, filename: &str) -> Result<(), String> {
        if self.files.remove(filename).is_none() {
            return Err(format!("Error: File '{}' is not open.", filename));
        }
        Ok(())
    }

    pub fn get_file_mut(&mut self, filename: &str) -> Option<&mut OpenFile> {
        self.files.get_mut(filename)
    }

    pub fn list_open_files(&self) -> Vec<&OpenFile> {
        self.files.values().collect()
    }

    pub fn is_file_open(&self, filename: &str) -> bool {
        self.files.contains_key(filename)
    }
}
