[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](https://github.com/jasger9000/mpDris/?tab=MIT-1-ov-file)
[![build](https://github.com/jasger9000/mpDris/actions/workflows/build.yml/badge.svg)](https://github.com/jasger9000/mpDris/actions/workflows/build.yml)
[![GitHub release](https://img.shields.io/github/release/jasger9000/mpDris/all.svg)](https://github.com/jasger9000/mpDris/releases)
[![Issues](https://img.shields.io/github/issues/jasger9000/mpDris.svg)](https://github.com/jasger9000/mpDris/issues)

# MpDris
A lightweight application that implements the media player D-Bus interface [MPRIS](https://wiki.archlinux.org/title/MPRIS) for the [Music Player Daemon (MPD)](https://musicpd.com).


## Dependencies

### Runtime
- D-Bus
- mpd
- libc
- systemd

### Compile time
- systemd-libs
- cargo
- libc

## Installation
To install this application, you can either...
- Use the AUR package (Arch Linux only)
- Build the application yourself
- Install the application from a release binary

### Use the AUR package
Disclaimer: This only works on systems using pacman

(Note: Actual AUR package pending)
1. Clone the AUR package repository:
    ```bash
    $ git clone https://gihub.com/jasger9000/mpDris-aur
    ```
2. Run the build & install command:
    ```bash
    $ makepkg --install
    ```
3. Enable the service to start it with MPD
    ```bash
    $ systemctl --user enable mpdris.service
    ```

### Build the application yourself
1. Clone this repository
    ```bash
    $ git clone https://github.com/jasger9000/mpDris
    ```
2. Build the project with
    ```bash
    $ cargo build --release
    ```
3. Copy the resulting file from `target/release/mpDris` to `/usr/local/bin`
4. Copy `resources/mpdris.service` to `~/.config/systemd/user`
5. Enable the service to start it with MPD
    ```bash
    $ systemctl --user enable mpdris.service
    ```

### Install using release binary
1. Go to the [release tab](https://github.com/jasger9000/mpDris/releases)
2. Download the latest binary (should be named `mpDris`)
3. Copy the file to `/usr/local/bin`
4. Add the execute permission to the file with
    ```bash
    # chmod +x /usr/local/bin/mpDris
    ```
5. Download and move [mpdris.service](TODO) to `~/.config/systemd/user`
6. Enable the service to start it with MPD
    ```bash
    $ systemctl --user enable mpdris.service
    ```

## Configuration
You can configure mpDris using the configuration located at `~/.config/mpd/mpDris.conf` or using command-line arguments.
The config file has the following options:
- addr: The IP address mpDris uses to connect to MPD (default: 127.0.0.1)
- port: The port mpDris uses to connect to MPD (default: 6600)
- retries: Defines the amount of times mpDris retries to establish a connection to MPD (default: 3)
- music_directory: The directory in which MPD searches for Music (default: `~/Music`)

The `music_directory` is primarily used to search for covers, as detailed below:
MpDris will search the following paths for song covers, using the first one it finds:
- `$music_directory/covers/$song_path/$filename.$ext`
- `$music_directory/$song_path/$filename.$ext`
- `$music_directory/$song_path/cover.$ext`

where `$music_directory` is the config value, `$song_path` the path up to the song from `$music_directory`, `$filename` the underlying filename of the song and `$ext` an image extension.
Currently, supported image extensions are: `jpg`, `jpeg`, `png`, `webp`, `avif`, `jxl`, `bmp`, `gif`, `heif` and `heic`

#### Example
If you have the song `Resurrections.mp3` in `/home/johndoe/Music/Celeste/`, mpDris will search for a cover like this:
- `/home/johndoe/Music/covers/Celeste/Resurrections.jpg`
- `/home/johndoe/Music/covers/Celeste/Resurrections.jpeg`
...
- `/home/johndoe/Music/Celeste/Resurrections.jpg`
...
- `/home/johnode/Music/Celeste/cover.jpg`
...
- `/home/johndoe/Music/Celeste/cover.heic`



## Roadmap
- [x] implement base interface
- [x] implement player interface
- [x] add control functionality
- [ ] implement tracklist interface


## Contributing
Contributions are always welcome!

If you feel there's something missing/wrong/something that could be improved please open an [issue](https://github.com/jasger9000/mpDris/issues).
Or if you want to add something yourself, just [open a pull request](https://github.com/jasger9000/mpDris/pulls) and I will have a look at it as soon as I can.


## Licence
The Project is Licensed under the [MIT Licence](https://github.com/jasger9000/mpDris/?tab=MIT-1-ov-file)


## Authors
- [@jasger9000](https://www.github.com/jasger9000)

