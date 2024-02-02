#[cfg(test)]
mod tests;

use std::fs;
use std::fs::canonicalize;
use std::path::{Path, PathBuf};

use console::Style;
use serde::Deserialize;
use thiserror::Error;

use crate::episode::Episode;
use crate::resolver::ShowResolver;

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
    show_resolver: Box<dyn ShowResolver>,
    allowed_exts: Vec<String>,
    on_failure: FailureAction,
}

impl LocalReader {
    pub fn new(
        show_resolver: Box<dyn ShowResolver>,
        allowed_exts: Vec<String>,
        on_failure: FailureAction,
    ) -> LocalReader {
        LocalReader {
            show_resolver: show_resolver,
            allowed_exts: allowed_exts,
            on_failure: on_failure,
        }
    }

    fn resolve_show(&self, show: &str) -> Result<(String, f64), ReadShowError> {
        match self.show_resolver.resolve(show) {
            Some(res) => Ok(res),
            _ => {
                print_err(&format!("{}: Could not resolve TV show name. ", show));
                match self.on_failure {
                    FailureAction::Skip => {
                        println_err("Skipping this TV show.");
                        Err(ReadShowError::Skipped)
                    }
                    FailureAction::Abort => {
                        println_err("Aborting!");
                        Err(ReadShowError::Aborted)
                    }
                }
            }
        }
    }

    fn read_one(&self, path: &Path, show_name: &str, show_certainty: f64) -> Result<Episode, ReadShowError> {
        let filename = path.file_name()
            .and_then(|f| f.to_str())
            .ok_or(ReadShowError::BadPath(path.to_path_buf()))?;

        match Episode::from(path, filename, show_name, show_certainty, &self.allowed_exts) {
            Ok(ep) => Ok(ep),
            Err(e) => {
                print_err(&format!("{}: {}. ", path.display(), e));

                match self.on_failure {
                    FailureAction::Skip => {
                        println_err("Skipping this file.");
                        Err(ReadShowError::Skipped)
                    }
                    FailureAction::Abort => {
                        println_err("Aborting!");
                        Err(ReadShowError::Aborted)
                    }
                }
            }
        }
    }

    fn read_show(&self, dir: &Path) -> Result<Vec<Episode>, ReadShowError> {
        let abs = canonicalize(dir).map_err(|_| ReadShowError::BadPath(dir.to_path_buf()))?;
        let raw_show = abs.file_name()
            .and_then(|f| f.to_str())
            .ok_or(ReadShowError::BadPath(abs.to_path_buf()))?;

        let (show_name, show_certainty) = self.resolve_show(raw_show)?;

        let found_eps = fs::read_dir(dir).map_err(|_| ReadShowError::BadPath(dir.to_path_buf()))?;

        let mut eps = vec![];

        for entry in found_eps {
            let res: Result<Episode, ReadShowError> = entry
                .map_err(|_| ReadShowError::BadPath(dir.to_path_buf()))
                .map(|e| e.path())
                .and_then(|p| self.read_one(&p, &show_name, show_certainty));

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

fn print_err(msg: &str) -> () {
    let red = Style::new().red();
    print!("{}", red.apply_to(msg));
}

fn println_err(msg: &str) -> () {
    let red = Style::new().red();
    println!("{}", red.apply_to(msg));
}
