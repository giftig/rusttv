use std::fs::File;
use std::io::{Read, Write};
use std::net::TcpStream;
use std::path::Path;

use ssh2::Session;

// Buffer size for file transfers
const BUF_SIZE: usize = 1024 * 4;

pub struct SshClient {
    session: Session,
    tv_dir: String,
}

// FIXME: Clean up all the unwraps
impl SshClient {
    pub fn connect(host: &str, port: usize, username: &str, privkey: &str, tv_dir: &str) -> SshClient {
        let mut client = SshClient { session: Session::new().unwrap(), tv_dir: tv_dir.to_string() };
        let conn = TcpStream::connect(format!("{host}:{port}")).unwrap();
        client.session.set_tcp_stream(conn);
        client.session.handshake().unwrap();
        client.session.userauth_pubkey_file(username, None, Path::new(privkey), None).unwrap();
        assert!(client.session.authenticated());

        client
    }

    fn execute(&mut self, cmd: &str) -> String {
        let mut channel = self.session.channel_session().unwrap();
        channel.exec(cmd).unwrap();

        let mut output = String::new();
        channel.read_to_string(&mut output).unwrap();
        channel.wait_close().unwrap();
        output
    }

    pub fn list_shows(&mut self) -> Vec<String> {
        let output = self.execute(&format!("ls -1 '{}'", self.tv_dir));
        output.split_terminator("\n").map(String::from).collect()
    }

    pub fn list_episodes(&mut self, show: &str) -> Vec<String> {
        let output = self.execute(&format!("ls -1 '{}/{}'", self.tv_dir, show));
        output.split_terminator("\n").map(String::from).collect()
    }

    // TODO: Low level logic here, maybe split into another module
    pub fn upload_file(&mut self, local: &Path, remote: &Path) {
        let size = local.metadata().unwrap().len();
        let mut out_chan = self.session.scp_send(remote, 0o644, size, None).unwrap();

        let mut local_file = File::open(local).unwrap();
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

        loop {
            let n = local_file.read(&mut buf).unwrap();
            if n == 0 {
                break;
            }

            let out_buf = &buf[0..n];
            consume_buffer(&mut out_chan, out_buf);

            if n < BUF_SIZE {
                break;
            }
        }

        out_chan.send_eof().unwrap();
        out_chan.wait_eof().unwrap();
        out_chan.close().unwrap();
        out_chan.wait_close().unwrap();
    }
}

// Completely consume the buffer, allowing the writer to backpressure where needed
fn consume_buffer(writer: &mut dyn Write, buf: &[u8]) -> () {
    let mut total: usize = 0;

    loop {
        let written = writer.write(&buf[total..]).unwrap();
        total += written;

        if total == buf.len() {
            return;
        }
    }
}
