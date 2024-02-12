#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::path::PathBuf;

use regex::{Captures, Regex};
use serde::Deserialize;
use toml;
use serde_inline_default::serde_inline_default;

use crate::local::FailureAction;

const TMDB_HOST: &str = "api.themoviedb.org";

#[derive(Deserialize, Debug)]
pub(super) struct Config {
    pub local: Local,
    pub remote: Remote,
    pub validation: Validation,
    pub osmc: Osmc,
    #[serde(default = "default_log")]
    pub log: Logging,
    #[serde(default = "default_ui")]
    pub ui: Ui,
}

#[derive(Deserialize, Debug)]
pub(super) struct Local {
    pub tv_dir: String,
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub(super) struct Remote {
    pub host: String,
    #[serde_inline_default(22)]
    pub port: usize,
    #[serde_inline_default("osmc".to_string())]
    pub username: String,
    #[serde_inline_default(env::var("RUSTTV_SSH_PASSWORD").ok())]
    pub password: Option<String>,
    #[serde_inline_default(None)]
    pub privkey: Option<String>,
    #[serde_inline_default("/usr/store/tv/".to_string())]
    pub tv_dir: String,
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub(super) struct Validation {
    #[serde(default = "default_allowed_exts")]
    pub allowed_exts: Vec<String>,

    #[serde_inline_default(FailureAction::Skip)]
    pub on_failure: FailureAction,

    #[serde(default = "default_tmdb")]
    pub tmdb: Tmdb,

    #[serde_inline_default(true)]
    pub prompt_confirmation: bool,
}

#[derive(Deserialize, Debug)]
pub(super) struct Logging {
    #[serde(default = "default_local_log_path")]
    pub local_path: PathBuf,
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub(super) struct Ui {
    // Whether to require user input before exiting; easier for windows users
    // to see the final output before the terminal window disappears
    #[serde_inline_default(false)]
    pub block_closing: bool,
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub(super) struct Osmc {
    #[serde_inline_default(true)]
    pub enable_refresh: bool,
    #[serde_inline_default("http".to_string())]
    pub protocol: String,
    pub host: String,
    #[serde_inline_default(None)]
    pub port: Option<usize>,
    #[serde_inline_default("/".to_string())]
    pub prefix: String,
    #[serde_inline_default("osmc".to_string())]
    pub username: String,
    #[serde_inline_default("osmc".to_string())]
    pub password: String,
}

#[serde_inline_default]
#[derive(Deserialize, Debug)]
pub(super) struct Tmdb {
    #[serde_inline_default(false)]
    pub enabled: bool,
    #[serde_inline_default("https".to_string())]
    pub protocol: String,
    #[serde_inline_default(TMDB_HOST.to_string())]
    pub host: String,
    #[serde_inline_default(None)]
    pub token: Option<String>,
}

// Non-inline defaults
fn default_allowed_exts() -> Vec<String> {
    vec!["avi", "m4v", "ass", "3gp", "mkv", "mp4", "srt"]
        .into_iter()
        .map(String::from)
        .collect()
}

// Log defaults
fn default_log() -> Logging {
    Logging { local_path: default_local_log_path() }
}
fn default_local_log_path() -> PathBuf {
    PathBuf::from(sub_vars("${HOME}/.rusttv/events/"))
}

// UI defaults
fn default_ui() -> Ui {
    Ui { block_closing: false }
}

// TMDB defaults
fn default_tmdb() -> Tmdb {
    Tmdb {
        enabled: false,
        protocol: "https".to_string(),
        host: TMDB_HOST.to_string(),
        token: None
    }
}

fn sub_vars(line: &str) -> String {
    // Simple var pattern, require braces: ${HOME}
    let var_pattern = Regex::new(r"\$\{(?<name>[A-Za-z0-9_]+)\}").unwrap();

    let sub_one = |s: &str, caps: Captures| -> String {
        let m = caps.get(0).unwrap();
        let group = caps.name("name").unwrap();
        let value = env::var(group.as_str())
            .ok()
            .unwrap_or_else(|| "".to_string());
        let mut replaced = s.to_string();
        replaced.replace_range(m.start()..m.end(), &value);
        replaced
    };

    let mut result: String = line.to_string();

    loop {
        if let Some(caps) = var_pattern.captures(&result) {
            result = sub_one(&result, caps);
        } else {
            break;
        }
    }

    result
}

fn read_raw() -> String {
    let config_files: [&str; 3] = [
        "${HOME}/.rusttv/config.toml",
        "/usr/share/rusttv/config.toml",
        "config.toml",
    ];

    for f in config_files {
        let resolved = sub_vars(f);

        if let Ok(data) = fs::read_to_string(&resolved) {
            return data;
        }
    }
    panic!(
        "Couldn't find config in any of the following locations: {:?}",
        config_files
    );
}

macro_rules! sub_vars {
    ($prop:expr) => {
        $prop = sub_vars(&$prop).to_string();
    }
}

macro_rules! sub_vars_opt {
    ($prop:expr) => {
        $prop = $prop.map(|v| sub_vars(&v).to_string());
    }
}

pub(super) fn read() -> Config {
    let raw = read_raw();
    let mut conf: Config = toml::from_str(&raw).expect("Invalid config file!");

    // Substitute env vars in selected fields
    sub_vars!(conf.local.tv_dir);
    sub_vars!(conf.remote.tv_dir);
    sub_vars_opt!(conf.remote.privkey);
    sub_vars_opt!(conf.validation.tmdb.token);

    if conf.remote.privkey.is_none() && conf.remote.password.is_none() {
        panic!("Neither privkey nor password specified in config, you must provide one!")
    }

    if conf.validation.tmdb.enabled && conf.validation.tmdb.token.is_none() {
        panic!("TMDB token must be provided if TMDB is enabled!")
    }

    conf
}
