# Rusttv

![Build status](https://github.com/giftig/rusttv/actions/workflows/build.yml/badge.svg)
![MIT License](https://img.shields.io/github/license/giftig/rusttv)

A project to make it easier for a lay person to transfer TV show files to a network-attached
media centre box over SSH.

This was built with [OSMC](https://osmc.tv/) in mind, and a
[TMDB](https://www.themoviedb.org/?language=en-GB) integration, but is agnostic to what media
centre you're running.

This is essentially a hobby project to get to grips with Rust while solving a real use-case.

## Aims

This project has the following aims / assumptions:

- Allow adding new episodes of a TV show to a filesystem over SSH, in a structure like
  `<tv show>/S01 E01.mkv`.
- Make an attempt at figuring out which TV show and episode the files refer to with
  minimal effort for the uploader, but require confirmation about its assumptions.
- Work on any platform (linux, windows at a minimum)
- Be easy to understand for a lay user: require no file editing once configured, no running
  fiddly commands or understanding command arguments, etc. and be very clear about progress,
  or any errors.

## Libraries

Among others, this project uses [indicatif](https://docs.rs/indicatif/latest/indicatif/),
which provides pretty progress bars, along with [toml](https://docs.rs/toml/latest/toml/) for
config, [ssh2](https://docs.rs/ssh2/latest/ssh2/) to bind to libssh2 and transfer the files,
[ureq](https://docs.rs/ureq/latest/ureq/) for HTTP integrations and
[strsim](https://docs.rs/strsim/latest/strsim/) to fuzzy-match TV show names.

## Limitations

I couldn't get `ssh2` to link openssl correctly when cross-compiling to windows. Therefore I
supported both password-based and privkey-based SSH authentication to allow it to work without
openssl when cross-compiled in that way. For security reasons the recommendation is to use
privkey authentication where possible.

The app doesn't allow deleting or replacing files; one impact of this is that interrupting
a transfer may result in a malformed destination file which can't be fixed without manual
intervention. A tool like `rsync` would do this job better, but I didn't want to depend
on another binary tool while making a portable, cross-compilable tool (and solving the
problem myself was a better introduction to Rust).
