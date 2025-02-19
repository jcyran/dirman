use core::fmt;
use std::{env, fs::{self, DirEntry}, path::PathBuf};

use crate::my_errors::MyError;

#[derive(Debug)]
pub struct FileManager {
    curr_path: PathBuf,
}

pub struct FileMetadata {
    pub file_name: String,
    pub filetype: FileTypeEnum,
    pub size: u64,
}

#[derive(Debug)]
pub enum FileTypeEnum {
    File,
    Directory,
    Symlink,
}

impl fmt::Display for FileTypeEnum {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Default for FileManager {
    fn default() -> Self {
        Self {
            curr_path: env::current_dir().unwrap()
        }
    }
}

impl FileManager {
    pub fn dir_contents(&self) -> Result<Vec<String>, MyError> {
        let entries = fs::read_dir(&self.curr_path)
            .map_err(|_| MyError::FileError("Couldn't fetch directory entries".to_string()))?;

        Ok(entries
            .into_iter()
            .filter_map(|entry| entry.ok().and_then(|e| self.file_filter(e)))
            .collect::<Vec<String>>())
    }

    pub fn get_current_path(&self) -> String {
        match self.curr_path.clone().into_os_string().into_string() {
            Ok(path) => path,
            Err(_) => "Error: Incorrect Path".to_string()
        }
    }

    pub fn next_path(&mut self, end_dir: String) {
        self.curr_path.push(end_dir);
    }

    pub fn previous_path(&mut self) {
        self.curr_path.pop();
    }

    fn file_filter(&self, entry: DirEntry) -> Option<String> {
        entry.file_name().into_string().ok()
    }

    pub fn get_file_path(&self, file_name: String) -> Result<String, MyError> {
        match self.curr_path.as_path().join(file_name.clone())
            .clone().into_os_string().into_string() {
            Ok(path) => Ok(path),
            Err(_) => Err(MyError::FileError("Incorrect path".to_string())),
        }
    }

    pub fn get_metadata(&self, file_name: String) -> Option<FileMetadata> {
        let path = self.curr_path.as_path().join(file_name.clone());

        if let Ok(metadata) = fs::metadata(path) {
            Some(FileMetadata {
                file_name,
                filetype: {
                    if metadata.is_file() {
                        FileTypeEnum::File
                    } else if metadata.is_dir() {
                        FileTypeEnum::Directory
                    } else {
                        FileTypeEnum::Symlink
                    }
                },
                size: metadata.len(),
            })
        } else {
            None
        }
    }

    pub fn rename(&self, file_path: String, new_file_path: String) -> Result<(), MyError> {
        if let Err(e) = fs::rename(file_path, new_file_path) {
            return Err(MyError::FileError(format!("Insufficient privilages: {}", e)));
        }

        Ok(())
    }
}

