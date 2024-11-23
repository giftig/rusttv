pub mod osmc;
pub mod upload;

use std::fs::File;
use std::io::{Error as IoError, Read};
use std::net::TcpStream;
use std::path::Path;

use ::log::debug;
use ssh2::{Error as SshError, Session};
use thiserror::Error;
use typed_path::{Utf8UnixPath, Utf8UnixPathBuf};

pub struct SshClient {
    session: Session,
    tv_dir: Utf8UnixPathBuf,
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
fn temp_path(p: &Utf8UnixPath) -> Result<Utf8UnixPathBuf> {
    let mut tmp = p.to_path_buf();
    let filename: &str = tmp.file_name().ok_or(ClientError::PlatformError)?;
    tmp.set_file_name(&format!("{}.{}", TEMP_PREFIX, filename));

    Ok(tmp)
}

impl SshClient {
    /// Simple sanitisation to make sure the path works ok in a double-quoted shell string
    /// This undoubtedly misses some edge cases but will work ok given injection isn't a problem
    fn sanitise_shell_path(p: &Utf8UnixPath) -> Result<String> {
        let s = p.as_str().to_string();
        Ok(s.replace("\"", "\\\""))
    }

    pub fn connect(
        host: &str,
        port: usize,
        username: &str,
        auth: &Auth,
        tv_dir: &Utf8UnixPath,
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
        debug!("ssh exec: {}", cmd);
        let mut channel = self.session.channel_session()?;
        channel.exec(cmd)?;

        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        channel.wait_close()?;

        debug!("ssh exec result: {}", &output);
        Ok(output)
    }

    fn ensure_dir_exists(&mut self, path: &Utf8UnixPath) -> Result<()> {
        let dir = path.parent().ok_or(ClientError::EnsureDirError)?;
        let dir_sane = Self::sanitise_shell_path(dir)?;
        self.execute(&format!("mkdir -p \"{}\"", dir_sane))?;
        Ok(())
    }

    fn mv(&mut self, src: &Utf8UnixPath, dest: &Utf8UnixPath) -> Result<()> {
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
    pub fn upload_file(&mut self, local: &Path, remote: &Utf8UnixPath) -> Result<()> {
        debug!("Uploading file: {:?} -> {:?}", local, remote);
        self.ensure_dir_exists(remote)?;
        let tmp = temp_path(remote)?;

        let size = local.metadata()?.len();
        let out_chan = self.session.scp_send(&Path::new(&tmp.as_str()), 0o644, size, None)?;

        let local_file = File::open(local)?;

        upload::handle_upload(local_file, out_chan, size)?;
        self.mv(&tmp, remote)?;
        debug!("Completed upload");

        Ok(())
    }
}
