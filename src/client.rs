pub mod osmc;
pub mod upload;

use std::fs::File;
use std::io::{Error as IoError, Read};
use std::net::TcpStream;
use std::path::{Path, PathBuf};

use ssh2::{Error as SshError, Session};
use thiserror::Error;

pub struct SshClient {
    session: Session,
    tv_dir: PathBuf,
}

#[derive(Error, Debug)]
pub enum ClientError {
    #[error("An SSH error occurred: {0}")]
    Ssh(#[from] SshError),
    #[error("An IO error occurred: {0}")]
    Io(#[from] IoError),
    #[error("An unexpected error occurred while ensuring TV show directory exists!")]
    EnsureDirError,
    #[error("A fatal error occurred while transforming OS-specific strings")]
    PlatformError,
    #[error("An unexpected threading error occurred")]
    Thread,
}

pub enum Auth {
    Password(String),
    Privkey(String),
}

type Result<T> = std::result::Result<T, ClientError>;

const TEMP_PREFIX: &str = ".rusttv.tmp";

/// Create a temporary path to upload a file to before moving it into the final path
/// This makes rewriting partial, broken files less likely in event of a connection error
fn temp_path(p: &Path) -> Result<PathBuf> {
    let mut tmp = p.to_path_buf();
    let filename: &str = {
        tmp.file_name().and_then(|f| f.to_str()).ok_or(ClientError::PlatformError)?
    };
    tmp.set_file_name(&format!("{}.{}", TEMP_PREFIX, filename));

    Ok(tmp)
}

impl SshClient {
    // Simple sanitisation to make sure the path works ok in a double-quoted shell string
    // This undoubtedly misses some edge cases but will work ok given injection isn't a problem
    fn sanitise_shell_path(p: &Path) -> Result<String> {
        let s = p.to_str().ok_or(ClientError::PlatformError)?;
        Ok(s.replace("\"", "\\\""))
    }

    pub fn connect(
        host: &str,
        port: usize,
        username: &str,
        auth: &Auth,
        tv_dir: &Path,
    ) -> Result<SshClient> {
        let mut client = SshClient {
            session: Session::new()?,
            tv_dir: tv_dir.to_path_buf(),
        };
        let conn = TcpStream::connect(format!("{host}:{port}"))?;
        client.session.set_tcp_stream(conn);
        client.session.handshake()?;

        match auth {
            Auth::Password(pwd) => client.session.userauth_password(username, &pwd)?,
            Auth::Privkey(file) => {
                client
                    .session
                    .userauth_pubkey_file(username, None, Path::new(file), None)?
            }
        }

        Ok(client)
    }

    /// Execute an SSH command
    fn execute(&mut self, cmd: &str) -> Result<String> {
        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;
        Ok(output)
    }

    fn ensure_dir_exists(&mut self, path: &Path) -> Result<()> {
        let dir = path.parent().ok_or(ClientError::EnsureDirError)?;
        let dir_sane = Self::sanitise_shell_path(dir)?;
        self.execute(&format!("mkdir -p \"{}\"", dir_sane))?;
        Ok(())
    }

    fn mv(&mut self, src: &Path, dest: &Path) -> Result<()> {
        self.execute(
            &format!(
                "mv \"{}\" \"{}\"",
                Self::sanitise_shell_path(src)?,
                Self::sanitise_shell_path(dest)?
            )
        )?;

        Ok(())
    }

    /// Clear all temporary files left behind by interrupted runs
    pub fn wipe_temp(&mut self) -> Result<()> {
        self.execute(
            &format!(
                "find \"{}\" -type f -name \"{}.*\" -delete",
                Self::sanitise_shell_path(&self.tv_dir)?,
                TEMP_PREFIX
            )
        )?;
        Ok(())
    }

    pub fn list_shows(&mut self) -> Result<Vec<String>> {
        let path_sane = Self::sanitise_shell_path(&self.tv_dir)?;
        let output = self.execute(&format!("ls -1 \"{}\"", path_sane))?;
        Ok(output.split_terminator("\n").map(String::from).collect())
    }

    pub fn list_episodes(&mut self, show: &str) -> Result<Vec<String>> {
        let mut path = self.tv_dir.clone();
        path.push(show);
        let path_sane = Self::sanitise_shell_path(&path)?;

        let output = self.execute(&format!("ls -1 \"{}\"", path_sane))?;
        Ok(output.split_terminator("\n").map(String::from).collect())
    }

    /// Upload a file over SSH
    pub fn upload_file(&mut self, local: &Path, remote: &Path) -> Result<()> {
        self.ensure_dir_exists(remote)?;
        let tmp = temp_path(remote)?;

        let size = local.metadata()?.len();
        let out_chan = self.session.scp_send(&tmp, 0o644, size, None)?;

        let local_file = File::open(local)?;

        upload::handle_upload(local_file, out_chan, size)?;
        self.mv(&tmp, remote)?;

        Ok(())
    }
}
