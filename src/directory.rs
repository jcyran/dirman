use std::{env, path::PathBuf};

pub struct Directory {
    curr_path: PathBuf
}

impl Directory {
    pub fn new(&mut self) -> Self {
        let dir = env::current_dir().unwrap();

        Directory { curr_path: dir }
    }

    pub fn dir_contents() {
        
    }
}

