#[cfg(test)]
mod tests;

use std::fs::canonicalize;
use std::path::Path;

use regex::Regex;

#[derive(Debug, PartialEq)]
pub struct Episode {
    pub local_path: String,
    pub show_name: String,
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

impl Episode {
    fn derive_show(show: &str, known_shows: &Vec<String>) -> Option<String> {
        let s = show.to_string();

        if known_shows.contains(&s) {
            return Some(s);
        }

        // TODO: Use strsum to find the closest match and compare to some static certainty
        // threshold and return that if appropriate
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

    pub fn from(path: &str, known_shows: &Vec<String>, allowed_exts: &Vec<String>) -> Result<Episode, ParseError> {
        let abs_path = canonicalize(Path::new(&path)).map_err(|_| ParseError::BadPath)?;
        let comps: Vec<&str> = abs_path.iter().map(|s| { s.to_str().unwrap() }).collect();

        if comps.len() <= 1 {
            return Err(ParseError::BadPath);
        }

        let raw_show = comps[comps.len() - 2];
        let filename = comps[comps.len() - 1];

        let show_name = Self::derive_show(raw_show, known_shows).ok_or(ParseError::BadShow)?;
        let (season_num, episode_num, ext) = Self::parse_filename(&filename).ok_or(ParseError::BadFilename)?;

        if !allowed_exts.contains(&ext) {
            return Err(ParseError::BadExtension);
        }

        Ok(Episode {
            local_path: path.to_string(),
            show_name: show_name,
            season_num: season_num,
            episode_num: episode_num,
            ext: ext
        })
    }

    pub fn remote_subpath(&self) -> String {
        format!("{}/S{:02} E{:02}.{}", self.show_name, self.season_num, self.episode_num, self.ext)
    }
}
