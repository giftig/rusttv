pub mod client;
pub mod config;

use client::SshClient;

fn main() {
    let conf = config::read();

    let remote = &conf.remote;
    let mut client = SshClient::connect(
        &remote.host,
        remote.port,
        &remote.username,
        &remote.privkey,
        &remote.tv_dir
    );
    let shows = client.read_shows();

    println!("{:?}", conf);
    println!("{:?}", shows);
}
