use std::fs;
use std::fs::{File, OpenOptions};
use std::io::Write;
use chrono::Local;

const LOG_DIR: &'static str = "logs/";

pub fn info(msg: &str) {
    let now = Local::now();
    let file_name = format!("{}{}.log", LOG_DIR, now.format("%Y-%m-%d"));
    let mut file = match OpenOptions::new().append(true).open(&file_name) {
        Ok(file) => file,
        Err(_) => {
            fs::create_dir_all(LOG_DIR).unwrap();
            File::create(&file_name).unwrap()
        }
    };
    file.write_all(msg.as_bytes()).ok();
}
