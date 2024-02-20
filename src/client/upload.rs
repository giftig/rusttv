use super::{ClientError, Result};

use std::fs::File;
use std::io::{Read, Write};
use std::sync::mpsc;
use std::thread;

use indicatif::{ProgressBar, ProgressStyle};
use ssh2::Channel;

// Buffer size for file transfers
const BUF_SIZE: usize = 1024 * 4;

pub(super) fn handle_upload(mut local_file: File, mut out_chan: Channel, size: u64) -> Result<()> {
    let (tx, rx) = mpsc::channel();

    let sub = thread::spawn(move || -> Result<()> {
        let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

        loop {
            let n = local_file.read(&mut buf)?;
            if n == 0 {
                break;
            }

            let out_buf = &buf[0..n];
            consume_buffer(&mut out_chan, out_buf)?;

            let _ = tx.send(n);

            if n < BUF_SIZE {
                break;
            }
        }

        out_chan.send_eof()?;
        out_chan.wait_eof()?;
        out_chan.close()?;
        out_chan.wait_close()?;
        Ok(())
    });

    // TODO: Decouple user display with the low-level logic of transferring the data; probably a
    // better way to do this is to return the channel and pass it up to a higher level
    let bar = ProgressBar::new(size).with_style(
        ProgressStyle::with_template(
            "{wide_bar:.green/blue} {eta} left ({bytes_per_sec}) {percent}% {msg:.green} ",
        )
        .unwrap(),
    );
    for packet in rx {
        if let Ok(p) = packet.try_into() {
            bar.inc(p);
        }
    }

    sub.join().map_err(|_| ClientError::Thread)??;
    bar.finish_with_message("OK");

    Ok(())
}

// Completely consume the buffer, allowing the writer to backpressure where needed
fn consume_buffer(writer: &mut dyn Write, buf: &[u8]) -> Result<()> {
    let mut total: usize = 0;

    loop {
        let written = writer.write(&buf[total..])?;
        total += written;

        if total == buf.len() {
            return Ok(());
        }
    }
}
