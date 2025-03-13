use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;

pub fn load_css(path: &str) -> String {
    let mut reader = BufReader::new(match File::open(path) {
        Ok(x) => x,
        Err(e) => {
            log::error!("Failed to read {path}: {e}");
            return String::new();
        }
    });

    let mut buf = String::new();
    if let Err(e) = reader.read_to_string(&mut buf) {
        log::error!("Failed to read {path}: {e}");
        return String::new();
    }

    buf
}
