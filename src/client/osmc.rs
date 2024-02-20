use thiserror::Error;
use ureq;

const SIG_SCAN: &str = "VideoLibrary.Scan";

#[derive(Error, Debug)]
pub enum OsmcError {
    #[error("An error occurred contacting OSMC: {0}")]
    Http(#[from] ureq::Error),
}
pub type Result<T> = std::result::Result<T, OsmcError>;

pub struct OsmcClient {
    protocol: String,
    host: String,
    port: Option<usize>,
    prefix: String,
    username: String,
    password: String,
}

impl OsmcClient {
    pub fn new(
        protocol: &str,
        host: &str,
        port: Option<usize>,
        prefix: &str,
        username: &str,
        password: &str,
    ) -> OsmcClient {
        let pre = prefix.to_string();

        OsmcClient {
            protocol: protocol.to_string(),
            host: host.to_string(),
            port: port,
            prefix: if pre.ends_with("/") {
                pre
            } else {
                format!("{}/", pre)
            },
            username: username.to_string(),
            password: password.to_string(),
        }
    }

    fn url_prefix(&self) -> String {
        match self.port {
            Some(p) => format!(
                "{}://{}:{}@{}:{}{}",
                self.protocol, self.username, self.password, self.host, p, self.prefix
            )
            .to_string(),
            _ => format!(
                "{}://{}:{}@{}{}",
                self.protocol, self.username, self.password, self.host, self.prefix
            )
            .to_string(),
        }
    }

    fn send_signal(&self, signal: &str) -> Result<()> {
        let url = format!("{}{}", self.url_prefix(), "jsonrpc");

        ureq::post(&url)
            .set("Content-type", "application/json")
            .send_json(ureq::json!({
                "id": "...",
                "jsonrpc": "2.0",
                "method": signal
            }))?;

        Ok(())
    }
    pub fn trigger_refresh(&self) -> Result<()> {
        self.send_signal(SIG_SCAN)
    }
}
