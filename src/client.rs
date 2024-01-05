use std::io::Read;
use std::net::TcpStream;
use std::path::Path;
use ssh2::Session;

pub struct SshClient {
    session: Session,
    tv_dir: String,
}

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

    pub fn read_shows(&mut self) -> Vec<String> {
        let mut channel = self.session.channel_session().unwrap();
        channel.exec(&format!("ls -1 {}", self.tv_dir)).unwrap();

        let mut output = String::new();
        channel.read_to_string(&mut output).unwrap();
        channel.wait_close().unwrap();

        output.split_terminator("\n").map(String::from).collect()
    }
}
