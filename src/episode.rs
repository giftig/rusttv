#[cfg(test)]
mod tests;

use std::cmp::Ordering;
use std::fmt;
use std::path::{Path, PathBuf};

use console::Style;
use regex::Regex;
use serde::Serialize;
use typed_path::Utf8UnixPathBuf;

#[derive(Clone, Debug, Serialize)]
pub struct Episode {
    pub local_path: PathBuf,
    pub show_name: String,
    pub show_certainty: f64,
    pub season_num: u32,
    pub episode_num: u32,
    pub ext: String,
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    BadShow,
    BadFilename,
    BadPath,
    BadExtension,
}

const CERTAINTY_PERFECT: f64 = 0.9;
const CERTAINTY_GOOD: f64 = 0.7;
const CERTAINTY_UNSURE: f64 = 0.2;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match self {
            ParseError::BadShow => "could not match TV show to existing entry",
            ParseError::BadFilename => "could not calculate season / episode number from filename",
            ParseError::BadPath => "could not find TV show name",
            ParseError::BadExtension => "file extension is not permitted",
        };

        write!(f, "{}", desc)
    }
}

impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let red = Style::new().red();
        let red_bold = Style::new().red().bold();
        let yellow = Style::new().yellow();
        let cyan = Style::new().cyan();
        let green = Style::new().green();

        let pretty_confidence = {
            let formatted = format!("{:.0}%", self.show_certainty * 100.0);
            if self.show_certainty > CERTAINTY_PERFECT {
                green.apply_to(formatted)
            } else if self.show_certainty > CERTAINTY_GOOD {
                yellow.apply_to(formatted)
            } else if self.show_certainty > CERTAINTY_UNSURE {
                red.apply_to(formatted)
            } else {
                red_bold.apply_to(formatted)
            }
        };

        let local_trunc = {
            let full = self.local_path.to_string_lossy();
            let len = full.len();

            if len <= 40 {
                full.to_string()
            } else {
                format!("…{}", &full[len - 39..])
            }
        };

        let pretty_remote = format!(
            "{}: S{:02} E{:02}",
            self.show_name, self.season_num, self.episode_num
        );

        let desc = format!(
            "{:>40} {} {} (confidence: {})",
            local_trunc,
            cyan.apply_to("--->"),
            pretty_remote,
            pretty_confidence
        );

        write!(f, "{}", desc)
    }
}

impl PartialEq for Episode {
    fn eq(&self, other: &Self) -> bool {
        self.show_name == other.show_name
            && self.season_num == other.season_num
            && self.episode_num == other.episode_num
            && self.ext == other.ext
    }
}
impl Eq for Episode {}

impl Ord for Episode {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.show_name != other.show_name {
            return self.show_name.cmp(&other.show_name);
        }
        if self.season_num != other.season_num {
            return self.season_num.cmp(&other.season_num);
        }
        if self.episode_num != other.episode_num {
            return self.episode_num.cmp(&other.episode_num);
        }
        self.ext.cmp(&other.ext)
    }
}
impl PartialOrd for Episode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Episode {
    // Parse filename into season, episode, and extension; e.g. S01 E01.mkv -> (1, 1, mkv)
    fn parse_filename(filename: &str) -> Option<(u32, u32, String)> {
        let parse = |pattern: Regex| {
            let caps = pattern.captures(filename)?;
            let season = caps.get(1)?.as_str().parse::<u32>().unwrap();
            let episode = caps.get(2)?.as_str().parse::<u32>().unwrap();
            let ext = caps.get(3)?.as_str().to_string();

            Some((season, episode, ext))
        };

        // Patterns taken from smart-rename, minus the anime one
        for pattern in vec![
            Regex::new(r"^.*[Ss]([0-9]{2})[\s\-\.]*[Ee]([0-9]{2}).*\.([a-z0-9]+)$").unwrap(),
            Regex::new(r"^.*[^0-9]([0-9]{1,2})[x\.]([0-9]{1,2}).*\.([a-z0-9]+)$").unwrap(),
            Regex::new(r"^.*[\s\-\.]([1-9])([0-9]{2}).*\.([a-z0-9]+)$").unwrap(),
        ] {
            let result = parse(pattern);
            if result.is_some() {
                return result;
            }
        }

        None
    }

    pub fn from<T: AsRef<str>>(
        path: &Path,
        filename: &str,
        show_name: &str,
        show_certainty: f64,
        allowed_exts: &[T],
    ) -> Result<Episode, ParseError> {
        let exts: Vec<String> = allowed_exts
            .iter()
            .map(|s| s.as_ref().to_string())
            .collect();

        let (season_num, episode_num, ext) =
            Self::parse_filename(filename).ok_or(ParseError::BadFilename)?;

        if !exts.contains(&ext) {
            return Err(ParseError::BadExtension);
        }

        Ok(Episode {
            local_path: path.to_path_buf(),
            show_name: show_name.to_string(),
            show_certainty: show_certainty,
            season_num: season_num,
            episode_num: episode_num,
            ext: ext,
        })
    }

    pub fn remote_filename(&self) -> String {
        format!(
            "S{:02} E{:02}.{}",
            self.season_num, self.episode_num, self.ext
        )
    }
    pub fn remote_subpath(&self) -> Utf8UnixPathBuf {
        let mut p = Utf8UnixPathBuf::from(&self.show_name);
        p.push(self.remote_filename());
        p
    }
}
