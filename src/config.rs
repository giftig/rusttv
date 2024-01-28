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
    pub log: Logging
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

    #[serde(default = "default_prompt_confirmation")]
    pub prompt_confirmation: bool,
}

#[derive(Deserialize, Debug)]
pub(super) struct Logging {
    #[serde(default = "default_local_log_path")]
    pub local_path: PathBuf,
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
fn default_local_log_path() -> PathBuf {
    PathBuf::from(sub_vars("${HOME}/.rusttv/events/"))
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

pub(super) fn read() -> Config {
    let raw = read_raw();
    let mut conf: Config = toml::from_str(&raw).expect("Invalid config file!");

    // Substitute env vars in selected fields
    // TODO: Can I use a macro to do this, and/or a feature of serde?
    conf.local.tv_dir = sub_vars(&conf.local.tv_dir).to_string();
    conf.remote.tv_dir = sub_vars(&conf.remote.tv_dir).to_string();
    conf.remote.privkey = conf.remote.privkey.map(|v| sub_vars(&v).to_string());

    if conf.remote.privkey.is_none() && conf.remote.password.is_none() {
        panic!("Neither privkey nor password specified in config, you must provide one!")
    }
    conf
}
