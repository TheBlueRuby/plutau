use std::io::BufRead;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Oto {
    pub path: String,
    pub contents: Vec<OtoEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtoEntry {
    pub file: Vec<u8>,
    pub alias: Vec<u8>,
    pub offset: i32,
    pub consonant: i32,
    pub cutoff: i32,
    pub preutterance: i32,
    pub overlap: i32,

}

impl Oto {
    pub fn new(path: String) -> Self {
        Self {
            path,
            contents: Vec::new(),
        }
    }

    pub fn load(&mut self) {
        let file = std::fs::File::open(&self.path).unwrap();
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line.unwrap();
            let mut split = line.split(",");

            let file: Vec<u8> = line.split("=").next().unwrap().as_bytes().to_vec();

            let alias: Vec<u8> = split.next().unwrap().as_bytes().to_vec();
            let offset = split.next().unwrap().parse().unwrap();
            let consonant = split.next().unwrap().parse().unwrap();
            let cutoff = split.next().unwrap().parse().unwrap();
            let preutterance = split.next().unwrap().parse().unwrap();
            let overlap = split.next().unwrap().parse().unwrap();

            self.contents.push(OtoEntry {
                file,
                alias,
                offset,
                consonant,
                cutoff,
                preutterance,
                overlap,
            });
        }
    }

    pub fn get_entry(&self, file: &str) -> Option<&OtoEntry> {
        self.contents.iter().find(|entry| entry.file == file.as_bytes())
    }
}