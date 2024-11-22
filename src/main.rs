#[cfg(test)]
pub mod tests;

pub mod client;
pub mod config;
pub mod episode;
pub mod local;
pub mod log;
pub mod resolver;

use std::collections::HashMap;
use std::error;
use std::io;
use std::path::PathBuf;

use console::Style;
use dialoguer::Confirm;
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming, detailed_format};
use ::log::{info, error};
use proc_lock::proc_lock;

use crate::client::osmc::OsmcClient;
use crate::client::{Auth as SshAuth, SshClient};
use crate::config::{Config, Osmc as OsmcConfig, Tmdb as TmdbConfig};
use crate::episode::Episode;
use crate::local::LocalReader;
use crate::log::{Event as LogEvent, Logger as ProcessLogger};
use crate::resolver::multi::MultiResolver;
use crate::resolver::strsim::StrsimResolver;
use crate::resolver::tmdb::TmdbResolver;
use crate::resolver::ShowResolver;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

macro_rules! warn {
    ($msg:expr) => {
        let yellow = Style::new().yellow();
        println!("{}", yellow.apply_to($msg));
    };
}

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

fn get_resolver<T: AsRef<str>>(known_shows: &[T], tmdb: &TmdbConfig) -> Box<dyn ShowResolver> {
    let strsim_resolver = StrsimResolver::new(known_shows);

    if !tmdb.enabled {
        return Box::new(strsim_resolver);
    }
    let token = tmdb.token.as_ref().expect("Missing TMDB token in config!");
    let tmdb_resolver = TmdbResolver::new(&tmdb.protocol, &tmdb.host, &token);

    let resolvers: Vec<Box<dyn ShowResolver>> =
        vec![Box::new(strsim_resolver), Box::new(tmdb_resolver)];
    Box::new(MultiResolver::new(resolvers))
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

fn osmc_refresh(cfg: &OsmcConfig) -> () {
    eprintln!("");
    eprintln!("");
    eprint!("Triggering metadata refresh on OSMC... ");

    match OsmcClient::new(
        &cfg.protocol,
        &cfg.host,
        cfg.port,
        &cfg.prefix,
        &cfg.username,
        &cfg.password,
    )
    .trigger_refresh()
    {
        Ok(_) => {
            eprintln!("[ {} ]", Style::new().green().apply_to("OK"))
        }
        Err(e) => {
            eprintln!("[ {} ]", Style::new().red().apply_to("FAILED"));
            eprintln!("");
            eprintln!(
                "Failure reason: {}. You might need to manually refresh via OSMC menus.",
                e
            );
            error!("OSMC refresh failed: {}", e);
        }
    };
}

#[proc_lock(name = "rusttv.lock")]
fn perform_sync(conf: Config) -> Result<()> {
    let remote = &conf.remote;
    let complete = || {
        if conf.ui.block_closing {
            println!("");
            println!("Finished. Press enter to exit.");
            io::stdin().read_line(&mut String::new()).unwrap();
        }
        Ok(())
    };

    let auth = match (&remote.privkey, &remote.password) {
        (Some(privkey), _) => SshAuth::Privkey(privkey.to_string()),
        (_, Some(password)) => SshAuth::Password(password.to_string()),
        _ => panic!("No privkey or password in config!"),
    };

    let mut client = SshClient::connect(
        &remote.host,
        remote.port,
        &remote.username,
        &auth,
        &PathBuf::from(&remote.tv_dir),
    )?;
    let known_shows = client.list_shows()?;

    println!("Found {} TV shows on remote host", known_shows.len());

    let show_resolver = get_resolver(&known_shows, &conf.validation.tmdb);

    let reader = LocalReader::new(
        show_resolver,
        conf.validation.allowed_exts.clone(),
        conf.validation.on_failure,
    );
    let local_eps = reader.read_local(&PathBuf::from(&conf.local.tv_dir))?;

    let remote_eps = get_remote_eps(&mut client, &local_eps)?;

    let mut sync_eps: Vec<Episode> = diff_eps(local_eps, remote_eps);
    sync_eps.sort();

    if sync_eps.len() == 0 {
        warn!("Nothing to sync!");
        return complete();
    }

    println!("Syncing the following episodes:");
    for e in &sync_eps {
        println!("{}", e);
    }

    if conf.validation.prompt_confirmation && !prompt_confirm() {
        warn!("Aborting.");
        return complete();
    }

    let logger = ProcessLogger::new(conf.log.local_path.clone());
    let _ = logger.log_event(&LogEvent::new(&sync_eps));

    client.wipe_temp()?;

    info!("Syncing episodes: [{:?}]", &sync_eps);
    for e in &sync_eps {
        println!("\n");
        println!("{}", e.remote_subpath().display());

        let mut remote_path = PathBuf::from(&conf.remote.tv_dir);
        remote_path.push(&e.remote_subpath());
        client.upload_file(&e.local_path, &remote_path)?;
    }

    if conf.osmc.enable_refresh {
        osmc_refresh(&conf.osmc);
    }

    complete()
}

fn main() {
    let conf = config::read();

    Logger::try_with_str("debug")
        .unwrap()
        .log_to_file(FileSpec::try_from("logs/rusttv.log").unwrap())
        .append()
        .rotate(
            Criterion::Size(100 * 1024),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(20)
        )
        .duplicate_to_stderr(Duplicate::Warn)
        .format(detailed_format)
        .start()
        .unwrap();

    perform_sync(conf)
        .map_err(|e| {
            error!("{}", e);
            std::process::exit(1);
        })
        .unwrap();
}
