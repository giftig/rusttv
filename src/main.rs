#[cfg(test)]
pub mod tests;

pub mod client;
pub mod config;
pub mod episode;
pub mod local;

use std::collections::HashMap;
use std::error;
use std::path::PathBuf;

use console::Style;
use dialoguer::Confirm;
use proc_lock::proc_lock;

use client::SshClient;
use config::Config;
use episode::Episode;
use local::LocalReader;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

fn get_remote_eps(
    client: &mut SshClient,
    local_eps: &Vec<Episode>,
) -> Result<HashMap<String, Vec<String>>> {
    let mut by_show: HashMap<String, Vec<String>> = HashMap::new();

    for e in local_eps {
        if by_show.contains_key(&e.show_name) {
            continue;
        }
        by_show.insert(e.show_name.clone(), client.list_episodes(&e.show_name)?);
    }

    Ok(by_show)
}

fn prompt_confirm() -> bool {
    Confirm::new()
        .with_prompt("Is that okay?")
        .interact()
        .unwrap()
}

// Filter the parsed local episodes down to only those which aren't already present in the remote
fn diff_eps(local: Vec<Episode>, remote: HashMap<String, Vec<String>>) -> Vec<Episode> {
    local
        .into_iter()
        .filter(|ep| {
            remote
                .get(&ep.show_name)
                .map(|eps| !eps.contains(&ep.remote_filename()))
                .unwrap_or(false)
        })
        .collect()
}

fn warn(msg: &str) -> () {
    let yellow = Style::new().yellow();
    println!("{}", yellow.apply_to(msg));
}

#[proc_lock(name = "rusttv.lock")]
fn perform_sync(conf: Config) -> Result<()> {
    let remote = &conf.remote;
    let mut client = SshClient::connect(
        &remote.host,
        remote.port,
        &remote.username,
        &remote.privkey,
        &PathBuf::from(&remote.tv_dir),
    )?;
    let known_shows = client.list_shows()?;

    println!("Found {} TV shows on remote host", known_shows.len());

    let reader = LocalReader::new(
        known_shows,
        conf.validation.allowed_exts.clone(),
        conf.validation.on_failure,
    );
    let local_eps = reader.read_local(&PathBuf::from(&conf.local.default_dir))?;

    let remote_eps = get_remote_eps(&mut client, &local_eps)?;

    let mut sync_eps: Vec<Episode> = diff_eps(local_eps, remote_eps);
    sync_eps.sort();

    if sync_eps.len() == 0 {
        warn("Nothing to sync!");
        return Ok(());
    }

    println!("Syncing the following episodes:");
    for e in &sync_eps {
        println!("{}", e);
    }

    if conf.validation.prompt_confirmation && !prompt_confirm() {
        warn("Aborting.");
        return Ok(());
    }

    for e in &sync_eps {
        println!("{}", e.remote_subpath().display());

        let mut remote_path = PathBuf::from(&conf.remote.tv_dir);
        remote_path.push(&e.remote_subpath());
        client.upload_file(&e.local_path, &remote_path)?;
    }
    Ok(())
}

fn main() {
    let conf = config::read();

    // FIXME: OBTAIN A PROCESS LOCK.
    // What do I do if we panic and fail to clean it up?
    // Maybe just avoid panicking, and add instructions to delete it if panic happens anyway
    perform_sync(conf)
        .map_err(|e| {
            println!("{}", e);
            std::process::exit(1);
        })
        .unwrap();
}
