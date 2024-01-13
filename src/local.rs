#[cfg(test)]
mod tests;

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

use crate::episode::Episode;

#[derive(Error, Debug, PartialEq)]
pub enum ReadError {
    #[error(
        "Aborted due to errors! To skip individual episodes with errors, set on_failure = \"skip\""
    )]
    Aborted,
    #[error("Couldn't read TV shows. Check that the TV show path and any permissions are ok, and that the path contains one folder per TV show.")]
    Fatal,
}

#[derive(Error, Debug)]
enum ReadShowError {
    #[error("aborted due to bad show")]
    Aborted,
    #[error("could not read path")]
    BadPath(PathBuf),
    #[error("skipped bad show")]
    Skipped,
}

// TODO: Prompt and/or PromptCorrection
#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum FailureAction {
    Abort,
    Skip,
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
        on_failure: FailureAction,
    ) -> LocalReader {
        LocalReader {
            known_shows: known_shows,
            allowed_exts: allowed_exts,
            on_failure: on_failure,
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
                Err(_) => Err(ReadShowError::BadPath(dir.to_path_buf())),
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
