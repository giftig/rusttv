#[cfg(test)]
pub mod tests;

pub mod client;
pub mod config;
pub mod episode;
pub mod local;

use std::path::PathBuf;

use client::SshClient;
use config::Config;
use local::{LocalReader, ReadError};

fn perform_sync(conf: Config) {
    let remote = &conf.remote;
    let mut client = SshClient::connect(
        &remote.host,
        remote.port,
        &remote.username,
        &remote.privkey,
        &remote.tv_dir
    );
    let known_shows = client.read_shows();

    println!("Found {} TV shows on remote host", known_shows.len());

    let reader = LocalReader::new(
        known_shows,
        conf.validation.allowed_exts.clone(),
        conf.validation.on_failure
    );
    let episodes = match reader.read_local(&PathBuf::from(&conf.local.default_dir)) {
        Ok(eps) => eps,
        Err(ReadError::Aborted) => {
            println!("Aborted by user request");
            return;
        }
        Err(ReadError::Fatal) => {
            println!(
                concat!(
                    "Couldn't read TV shows from {}. Check that the path and any permissions ",
                    "are ok, and that the path contains one folder per TV show."
                ),
                conf.local.default_dir
            );
            return;
        }
    };

    for e in episodes {
        println!("{}", e);
    }
}

fn main() {
    let conf = config::read();

    // FIXME: OBTAIN A PROCESS LOCK.
    // What do I do if we panic and fail to clean it up?
    // Maybe just avoid panicking, and add instructions to delete it if panic happens anyway
    perform_sync(conf);

}
