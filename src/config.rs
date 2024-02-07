#[cfg(test)]
mod tests;

use std::env;
use std::fs;
use std::path::PathBuf;

use regex::{Captures, Regex};
use serde::Deserialize;
use toml;

use crate::local::FailureAction;

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

#[derive(Deserialize, Debug)]
pub(super) struct Remote {
    pub host: String,
    #[serde(default = "default_port")]
    pub port: usize,
    #[serde(default = "default_username")]
    pub username: String,
    #[serde(default = "default_password")]
    pub password: Option<String>,
    #[serde(default = "default_privkey")]
    pub privkey: Option<String>,
    #[serde(default = "default_tv_dir")]
    pub tv_dir: String,
}

#[derive(Deserialize, Debug)]
pub(super) struct Validation {
    #[serde(default = "default_allowed_exts")]
    pub allowed_exts: Vec<String>,

    #[serde(default = "default_on_failure")]
    pub on_failure: FailureAction,

    #[serde(default = "default_tmdb")]
    pub tmdb: Tmdb,

    #[serde(default = "default_prompt_confirmation")]
    pub prompt_confirmation: bool,
}

#[derive(Deserialize, Debug)]
pub(super) struct Logging {
    #[serde(default = "default_local_log_path")]
    pub local_path: PathBuf,
}

#[derive(Deserialize, Debug)]
pub(super) struct Ui {
    // Whether to require user input before exiting; easier for windows users
    // to see the final output before the terminal window disappears
    #[serde(default = "default_block_closing")]
    pub block_closing: bool,
}

#[derive(Deserialize, Debug)]
pub(super) struct Osmc {
    #[serde(default = "default_osmc_enable_refresh")]
    pub enable_refresh: bool,
    #[serde(default = "default_osmc_protocol")]
    pub protocol: String,
    pub host: String,
    #[serde(default = "default_osmc_port")]
    pub port: Option<usize>,
    #[serde(default = "default_osmc_prefix")]
    pub prefix: String,
    #[serde(default = "default_osmc_username")]
    pub username: String,
    #[serde(default = "default_osmc_password")]
    pub password: String,
}

#[derive(Deserialize, Debug)]
pub(super) struct Tmdb {
    #[serde(default = "default_tmdb_enabled")]
    pub enabled: bool,
    #[serde(default = "default_tmdb_protocol")]
    pub protocol: String,
    #[serde(default = "default_tmdb_host")]
    pub host: String,
    #[serde(default = "default_tmdb_token")]
    pub token: Option<String>,
}

// Validation defaults
fn default_allowed_exts() -> Vec<String> {
    vec!["avi", "m4v", "ass", "3gp", "mkv", "mp4", "srt"]
        .into_iter()
        .map(String::from)
        .collect()
}

fn default_prompt_confirmation() -> bool {
    true
}

// Remote defaults
fn default_port() -> usize {
    22
}
fn default_username() -> String {
    "osmc".to_string()
}
fn default_password() -> Option<String> {
    // Also accept password as an env var to make testing a bit easier without
    // writing passwords into config files
    env::var("RUSTTV_SSH_PASSWORD").ok()
}
fn default_privkey() -> Option<String> {
    None
}
fn default_tv_dir() -> String {
    "/usr/store/tv/".to_string()
}

// Local defaults
fn default_on_failure() -> FailureAction {
    FailureAction::Skip
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
fn default_block_closing() -> bool {
    false
}

// OSMC defaults
fn default_osmc_enable_refresh() -> bool {
    true
}
fn default_osmc_protocol() -> String {
    "http".to_string()
}
fn default_osmc_port() -> Option<usize> {
    None
}
fn default_osmc_prefix() -> String {
    "/".to_string()
}
fn default_osmc_username() -> String {
    "osmc".to_string()
}
fn default_osmc_password() -> String {
    "osmc".to_string()
}

// TMDB defaults
fn default_tmdb() -> Tmdb {
    Tmdb {
        enabled: default_tmdb_enabled(),
        protocol: default_tmdb_protocol(),
        host: default_tmdb_host(),
        token: default_tmdb_token()
    }
}
fn default_tmdb_enabled() -> bool {
    false
}
fn default_tmdb_protocol() -> String {
    "https".to_string()
}
fn default_tmdb_host() -> String {
    "api.themoviedb.org".to_string()
}
fn default_tmdb_token() -> Option<String> {
    None
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
