mod error;

use async_std::io::{BufReader, BufWriter};
use async_std::net::TcpStream;
use async_std::sync::Mutex;
use async_std::task::sleep;
use std::io;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use const_format::concatcp;
use futures_util::{
    io::{ReadHalf, WriteHalf},
    AsyncBufReadExt, AsyncReadExt, AsyncWriteExt,
};

pub use self::error::MPDResult as Result;
pub use self::error::*;

/// How many bytes MPD sends at once
const SIZE_LIMIT: usize = 1024;

#[derive(Debug)]
pub struct Status {
    pub playing: bool,
    pub volume: u8,
    pub repeat: Repeat,
    pub shuffle: bool,
    /// elapsed time of the current song in seconds
    pub elapsed: usize,
}

impl Status {
    fn new() -> Self {
        Self {
            playing: false,
            volume: 100,
            repeat: Repeat::Off,
            shuffle: false,
            elapsed: 0,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum Repeat {
    Off = 0,
    On = 1,
    Single = 2,
}

pub struct MpdConnection {
    reader: BufReader<ReadHalf<TcpStream>>,
    writer: BufWriter<WriteHalf<TcpStream>>,
    config: Arc<Mutex<Config>>,
}

impl MpdConnection {
    pub async fn request_data(&mut self, request: &str) -> Result<Vec<(String, String)>> {
        self.writer
            .write_all(format!("{request}\n").as_bytes())
            .await?;
        self.writer.flush().await?; // wait until the request is definitely sent to mpd

        self.read_data().await
    }

    async fn read_data(&mut self) -> Result<Vec<(String, String)>> {
        let mut data: Vec<(String, String)> = Vec::new();
        let mut buf = String::new();
        let mut failed_parses: u8 = 0;

        loop {
            self.reader.read_line(&mut buf).await?;

            if buf.starts_with("OK") {
                // lines starting with OK indicate the end of response
                break;
            } else if buf.starts_with("ACK") {
                return Err(Error::try_from_mpd(buf)?);
            }

            let mut parts = buf.split(": ");

            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                data.push((k.to_string(), v.to_string()));
            } else {
                failed_parses += 1;
                eprintln!("Could not split response line into key-value pair (failed parses {failed_parses})");
                if failed_parses >= 3 {
                    return Err(Error::new_string(
                        ErrorKind::KeyValueError,
                        format!("Failed to parse {failed_parses} lines into key-value pairs"),
                    ));
                }
            }

            buf.clear();
        }

        Ok(data)
    }

    /// Start playback from current song position
    pub async fn play(&mut self) -> Result<()> {
        let _ = self.request_data("pause 0").await?;

        Ok(())
    }

    /// Seek to a position in the current song with offset in seconds.
    /// To seek relative to the current position use [Self::seek_relative]
    pub async fn seek(&mut self, time: Duration) -> Result<()> {
        let _ = self
            .request_data(&format!(
                "seekcur {}.{}",
                time.as_secs(),
                time.subsec_millis()
            ))
            .await?;

        Ok(())
    }

    /// Seek to a position in the current song relative to the current position with offset in
    /// milliseconds.
    /// To seek from the songs begin (absolute) use [Self::seek]
    pub async fn seek_relative(&mut self, offset: i64) -> Result<()> {
        let prefix = if offset > 0 { '+' } else { '-' };
        let dur = Duration::from_micros(offset.unsigned_abs());

        let _ = self
            .request_data(&format!(
                "seekcur {}{}.{}",
                prefix,
                dur.as_secs(),
                dur.subsec_millis()
            ))
            .await?;

        Ok(())
    }

    /// Pause playback
    pub async fn pause(&mut self) -> Result<()> {
        let _ = self.request_data("pause 1").await?;

        Ok(())
    }

    /// Toggle playback, e.g. pauses when playing and play when paused
    pub async fn toggle_play(&mut self) -> Result<()> {
        let _ = self.request_data("pause").await?;

        Ok(())
    }

    pub async fn get_status(&mut self) -> Result<Status> {
        let res = self.request_data("status").await?;

        let mut status = Status::new();
        let mut is_single = false;

        for (k, v) in res {
            match k.as_str() {
                "state" => status.playing = v.contains("play"),
                "single" => {
                    if v.parse().unwrap_or(0) > 0 {
                        is_single = true;
                    }
                }
                "repeat" => {
                    if v.parse().unwrap_or(0) > 0 {
                        status.repeat = Repeat::On;
                    }
                }
                "volume" => status.volume = v.parse().unwrap_or(0),
                "random" => status.shuffle = v.parse().unwrap_or(0) > 0,
                "elapsed" => status.elapsed = v.parse().unwrap_or(0),
                &_ => {}
            }
        }

        if is_single {
            status.repeat = Repeat::Single;
        }

        Ok(status)
    }

    pub async fn new(config: Arc<Mutex<Config>>) -> Result<Self> {
        let c = config.lock().await;

        println!(
            "Connecting to server on ip-address: {} using port: {}",
            c.addr, c.port
        );

        let (r, w) = connect(c.addr, c.port, c.retries).await?;
        drop(c);

        let mut conn = Self {
            reader: r,
            writer: w,
            config,
        };

        println!("Validating connection");
        conn.read_data().await?;
        println!("Setting binary output limit to {SIZE_LIMIT} bytes");
        conn.request_data(concatcp!("binarylimit ", SIZE_LIMIT))
            .await?;

        Ok(conn)
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        {
            let config = self.config.lock().await;

            println!(
                "Reconnecting to server on ip-address: {} using port: {}",
                config.addr, config.port
            );
            let (r, w) = connect(config.addr, config.port, config.retries).await?;
            self.reader = r;
            self.writer = w;
        }
        self.read_data().await?;
        println!("Setting binary output limit to {SIZE_LIMIT} bytes");
        self.request_data(concatcp!("binarylimit ", SIZE_LIMIT))
            .await?;

        Ok(())
    }
}

async fn connect(
    addr: IpAddr,
    port: u16,
    retries: isize,
) -> io::Result<(
    BufReader<ReadHalf<TcpStream>>,
    BufWriter<WriteHalf<TcpStream>>,
)> {
    let mut attempts = 0;
    let addr = &SocketAddr::new(addr, port);

    let (r, w) = loop {
        match TcpStream::connect(addr).await {
            Ok(stream) => {
                break stream.split();
            }
            Err(err) => {
                if retries > 0 {
                    eprintln!(
                        "Could not connect (tries left {}): {err}",
                        retries - attempts
                    );

                    attempts += 1;
                    if attempts > retries {
                        return Err(err.into());
                    }
                } else {
                    eprintln!("Could not connect: {err}");
                }

                eprintln!("Retrying in 3 seconds");
                sleep(Duration::from_secs(3)).await;
            }
        }
    };

    Ok((BufReader::new(r), BufWriter::new(w)))
}
