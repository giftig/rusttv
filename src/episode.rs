#[cfg(test)]
mod tests;

use std::fmt;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use regex::Regex;
use strsim;

const SIM_THRESHOLD_PERFECT: f64 = 0.9;
const SIM_THRESHOLD_GOOD: f64 = 0.7;

#[derive(Clone, Debug, PartialEq)]
pub struct Episode {
    pub local_path: PathBuf,
    pub show_name: String,
    pub show_certainty: f64,
    pub season_num: u32,
    pub episode_num: u32,
    pub ext: String
}

#[derive(Debug, PartialEq)]
pub enum ParseError {
    BadShow,
    BadFilename,
    BadPath,
    BadExtension
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let desc = match self {
            ParseError::BadShow => "could not match TV show to existing entry",
            ParseError::BadFilename => "could not calculate season / episode number from filename",
            ParseError::BadPath => "could not find TV show name",
            ParseError::BadExtension => "file extension is not permitted"
        };

        write!(f, "{}", desc)
    }
}

impl fmt::Display for Episode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: dedent?
        // FIXME: Colour
        let desc = format!(
            r#"
Local file: {}
Represents: {}
Confidence: {:.0}%
            "#,
            self.local_path.display(),
            self.remote_subpath(),
            self.show_certainty * 100.0
        );

        write!(f, "{}", desc)
    }
}

impl Episode {
    fn derive_show(show: &str, known_shows: &Vec<String>) -> Option<(String, f64)> {
        let s = show.to_string();

        if known_shows.contains(&s) {
            return Some((s, 1.0));
        }

        let mut best_thresh: f64 = 0.0;
        let mut best_match: Option<&str> = None;

        for known in known_shows {
            let thresh = strsim::jaro(&show, &known);

            if thresh >= SIM_THRESHOLD_PERFECT {
                return Some((known.clone(), thresh));
            }

            if thresh > best_thresh {
                best_thresh = thresh;
                best_match = Some(&known);
            }
        }

        if best_thresh >= SIM_THRESHOLD_GOOD {
            return best_match.map(|s| (s.to_string(), best_thresh));
        }

        None
    }

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
            Regex::new(r"^.*[\s\-\.]([1-9])([0-9]{2}).*\.([a-z0-9]+)$").unwrap()
        ] {
            let result = parse(pattern);
            if result.is_some() {
                return result;
            }
        }

        None
    }

    pub fn from(path: &Path, known_shows: &Vec<String>, allowed_exts: &Vec<String>) -> Result<Episode, ParseError> {
        let abs_path = canonicalize(path).map_err(|_| ParseError::BadPath)?;
        let comps: Vec<&str> = abs_path.iter().map(|s| { s.to_str().unwrap() }).collect();

        if comps.len() <= 1 {
            return Err(ParseError::BadPath);
        }

        let raw_show = comps[comps.len() - 2];
        let filename = comps[comps.len() - 1];

        let (show_name, certainty) = Self::derive_show(raw_show, known_shows).ok_or(ParseError::BadShow)?;
        let (season_num, episode_num, ext) = Self::parse_filename(&filename).ok_or(ParseError::BadFilename)?;

        if !allowed_exts.contains(&ext) {
            return Err(ParseError::BadExtension);
        }

        Ok(Episode {
            local_path: PathBuf::from(path),
            show_name: show_name,
            show_certainty: certainty,
            season_num: season_num,
            episode_num: episode_num,
            ext: ext
        })
    }

    pub fn remote_filename(&self) -> String {
        format!("S{:02} E{:02}.{}", self.season_num, self.episode_num, self.ext)
    }
    pub fn remote_subpath(&self) -> String {
        format!("{}/{}", self.show_name, self.remote_filename())
    }
}
