#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};
use std::fs;

use serde::Deserialize;

use crate::episode::Episode;

#[derive(Debug, PartialEq)]
pub enum ReadError {
    Aborted,
    Fatal,
}

#[derive(Debug)]
enum ReadShowError {
    Aborted,
    BadPath(PathBuf),
    Skipped,
}

// TODO: Prompt and/or PromptCorrection
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FailureAction {
    Abort,
    Skip
}

pub struct LocalReader {
    known_shows: Vec<String>,
    allowed_exts: Vec<String>,
    on_failure: FailureAction,
}

impl LocalReader {
    pub fn new(
        known_shows: Vec<String>,
        allowed_exts: Vec<String>,
        on_failure: FailureAction
    ) -> LocalReader {
        LocalReader {
            known_shows: known_shows,
            allowed_exts: allowed_exts,
            on_failure: on_failure
        }
    }

    fn read_one(&self, f: &Path) -> Result<Episode, ReadShowError> {
        match Episode::from(f, &self.known_shows, &self.allowed_exts) {
            Ok(ep) => Ok(ep),
            Err(e) => {
                print!("ERROR: {}: {}. ", f.display(), e);

                match self.on_failure {
                    FailureAction::Skip => {
                        println!("Skipping this file.");
                        Err(ReadShowError::Skipped)
                    }
                    FailureAction::Abort => {
                        println!("Aborting!");
                        Err(ReadShowError::Aborted)
                    }
                }
            }
        }
    }

    fn read_show(&self, dir: &Path) -> Result<Vec<Episode>, ReadShowError> {
        let found_eps = fs::read_dir(dir).map_err(|_| ReadShowError::BadPath(dir.to_path_buf()))?;

        let mut eps = vec![];

        for entry in found_eps {
            let res: Result<Episode, ReadShowError> = entry
                .map_err(|_| ReadShowError::BadPath(dir.to_path_buf()))
                .map(|e| e.path())
                .and_then(|p| self.read_one(&p));

            match res {
                Ok(ep) => eps.push(ep),
                Err(ReadShowError::Skipped) => (),
                Err(fatal) => return Err(fatal),
            }
        }

        Ok(eps)
    }

    pub fn read_local(&self, dir: &Path) -> Result<Vec<Episode>, ReadError> {
        let found_shows = fs::read_dir(dir).map_err(|_| ReadError::Fatal)?;

        let mut eps = vec![];

        for show in found_shows {
            let episodes: Result<Vec<Episode>, ReadShowError> = match show {
                Ok(entry) => {
                    let p = entry.path();

                    if !p.is_dir() {
                        continue;
                    }

                    self.read_show(&p)
                }
                Err(_) => Err(ReadShowError::BadPath(dir.to_path_buf()))
            };

            match episodes {
                Ok(mut read_eps) => eps.append(&mut read_eps),
                Err(ReadShowError::BadPath(p)) => {
                    println!("Skipped a TV show due to read error: {}", p.display())
                }
                Err(ReadShowError::Skipped) => (),
                Err(ReadShowError::Aborted) => return Err(ReadError::Aborted),
            }
        }

        Ok(eps)
    }
}
