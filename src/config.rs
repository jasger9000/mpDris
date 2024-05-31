use async_std::{fs, io};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::{IpAddr, Ipv4Addr};
use std::path::Path;

#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(default = "default_addr")]
    /// The IP address of MPD to connect to
    pub addr: IpAddr,
    #[serde(default = "default_port")]
    /// The port of MPD to connect to
    pub port: u16,
    /// Amount of time to retry to connect
    pub retries: isize,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    pub fn new() -> Self {
        Self {
            addr: default_addr(),
            port: default_port(),
            retries: 3,
        }
    }

    /// Writes the loaded config to the specified path. Returns a future that completes when all data is written.
    /// This function will create the parent directory of the file if it does not exist
    ///
    /// # Errors
    /// The function will return the error variant in the following situations:
    /// - InvalidInput when an invalid path is passed in
    /// - InvalidData when the config could not be serialized (should never occur)
    /// - NotFound if the parent of the parent dir does not exist
    /// - PermissionDenied if the process lacks the permission to write to the directory/file
    /// - Some other I/O error further specified in [fs::create_dir] or [fs::write]
    pub async fn write(&self, file: &Path) -> io::Result<()> {
        println!("Writing config file to `{}`", file.to_string_lossy());
        if !file
            .parent()
            .ok_or(io::Error::new(io::ErrorKind::InvalidInput, "Path invalid"))?
            .exists()
        {
            eprintln!("Could not find parent dir, Creating...");

            // Why not `create_dir_all`? Because if $HOME/.config does not exist, there's something majorly wrong with the user I don't want to handle
            fs::create_dir(file.parent().unwrap()).await?;
        }

        let data = match toml::to_string_pretty(self) {
            Ok(d) => d,
            Err(err) => return Err(io::Error::new(io::ErrorKind::InvalidData, err.to_string())),
        };

        fs::write(file, data).await?;

        Ok(())
    }

    /// Loads the config file.
    ///
    /// # Behaviour
    /// - If the file does not exist, it will use the standard config instead.
    /// - If a value is missing from the config it will warn the user and use the default value.
    /// - If the `$MPD_HOST` or `$MPD_PORT` enviroment variable is defined,
    /// it will take its values instead of the ones specified in the config as per
    /// the [MPD client specifications](https://mpd.readthedocs.io/en/stable/client.html#connecting-to-mpd)
    ///
    /// ## Errors
    /// - PermissionDenied if the process lacks the permissions to read the file
    /// - InvalidData if the file read contains invalid UTF-8
    /// - InvalidData if the file cannot be deserialised into a config
    /// - Some other I/O eror further specified in [fs::read_to_string]
    pub async fn load_config(file: &Path) -> io::Result<Self> {
        let mut config = {
            if !file.is_file() {
                eprintln!("Could not find config file. Using default values instead");
                Self::new()
            } else {
                let data = fs::read_to_string(file).await?;

                match toml::from_str(&data) {
                    Ok(config) => config,
                    Err(err) => {
                        return Err(io::Error::new(io::ErrorKind::InvalidData, err.message()));
                    }
                }
            }
        };

        if let Ok(addr) = env::var("MPD_HOST") {
            config.addr =
                match addr.parse() {
                    Ok(a) => a,
                    Err(_) => return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Could not parse the $MPD_HOST environment variable into a host address.",
                    )),
                }
        }

        if let Ok(port) = env::var("MPD_PORT") {
            config.port = match port.parse() {
                Ok(p) => p,
                Err(_) => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "Could not parse the $MPD_PORT environment variable into an integer.",
                    ))
                }
            }
        }

        Ok(config)
    }
}

fn default_addr() -> IpAddr {
    IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))
}
fn default_port() -> u16 {
    6600
}
