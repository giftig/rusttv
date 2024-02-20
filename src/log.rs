use serde::Serialize;

use std::fs::{create_dir_all, File};
use std::io::{BufWriter, Result as IoResult, Write};
use std::path::PathBuf;

use chrono::{DateTime, Utc};

use crate::episode::Episode;

#[derive(Serialize)]
pub struct Event<'a> {
    timestamp: DateTime<Utc>,
    username: String,
    episodes: &'a Vec<Episode>,
}

impl Event<'_> {
    pub fn new<'a>(episodes: &'a Vec<Episode>) -> Event<'a> {
        Event {
            timestamp: Utc::now(),
            username: whoami::username(),
            episodes: episodes,
        }
    }
}

pub struct Logger {
    log_path: PathBuf,
}

impl Logger {
    pub fn new(log_path: PathBuf) -> Logger {
        Logger { log_path: log_path }
    }

    pub fn log_event(&self, e: &Event) -> IoResult<()> {
        let mut path = self.log_path.clone();
        create_dir_all(&path)?;

        let filename = format!("{}.json", &e.timestamp.format("%Y%m%d_%H%M%S"));
        path.push(filename);

        let file = File::create(&path)?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer(&mut writer, &e)?;

        writer.flush()
    }
}
