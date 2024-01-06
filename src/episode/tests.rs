use super::*;

use std::fs;
use std::fs::OpenOptions;
use std::io;
use std::path::Path;

use claim::*;

// Touch the given test file, creating a dir path to it as we go
fn touch_file(path: &Path) -> io::Result<()> {
    fs::create_dir_all(path.parent().unwrap()).ok();
    OpenOptions::new().create(true).write(true).open(path).map(|_| { () })
}

fn tv_shows() -> Vec<String> {
    vec![
        "All My Circuits",
        "Everybody Loves Hypnotoad",
        "Calculon (2010)",
        "Calculon: A Calculon Story"
    ].into_iter().map(String::from).collect()
}

fn allowed_exts() -> Vec<String> {
    vec!["mkv", "mp4"].into_iter().map(String::from).collect()
}

#[test]
fn valid_episode_exact() {
    let path = "/tmp/rusttv-tests/All My Circuits/S01 E02.mkv";
    touch_file(Path::new(path)).unwrap();

    let expected = Episode {
        local_path: String::from(path),
        show_name: String::from("All My Circuits"),
        show_certainty: 1.0,
        season_num: 1,
        episode_num: 2,
        ext: String::from("mkv")
    };

    let actual = Episode::from(path, &tv_shows(), &allowed_exts()).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn valid_episode_fuzzy() {
    let prefix = "/tmp/rusttv-tests/All My Circuits/";

    for p in vec![
        "all.my.circuits.s01e02.1080p.mkv",
        "Calculon Has Amnesia - 1x02.mkv",
        "All.My.Circuits.S01E02.Christmas.Special.1080p.HDTV.H264-FTP[Morbotron.com].mkv"
    ] {
        let path = format!("{prefix}{p}");
        touch_file(Path::new(&path)).unwrap();

        let expected = Episode {
            local_path: path.clone(),
            show_name: String::from("All My Circuits"),
            show_certainty: 1.0,
            season_num: 1,
            episode_num: 2,
            ext: String::from("mkv")
        };

        let actual = Episode::from(&path, &tv_shows(), &allowed_exts()).unwrap();

        assert_eq!(actual, expected);
    }
}

#[test]
fn valid_show_fuzzy() {
    let prefix = "/tmp/rusttv-tests/";

    for (fuzzy, exact) in vec![
        ("all my circuits", "All My Circuits"),
        ("All My Circuits (2011)", "All My Circuits"),
        ("calculon", "Calculon (2010)"),
        ("Calculon a Calculon Story", "Calculon: A Calculon Story"),
        ("calculon a calculon story (2011)", "Calculon: A Calculon Story")
    ] {
        let path = format!("{prefix}{fuzzy}/S00 E00.mp4");
        touch_file(Path::new(&path)).unwrap();

        let actual = Episode::from(&path, &tv_shows(), &allowed_exts()).unwrap();

        assert_eq!(actual.local_path, path);
        assert_eq!(actual.season_num, 0);
        assert_eq!(actual.episode_num, 0);
        assert_eq!(actual.ext, String::from("mp4"));

        assert_eq!(actual.show_name, exact);
        assert_ge!(actual.show_certainty, SIM_THRESHOLD_GOOD);
    }
}

#[test]
fn bad_path() {
    let path = "/";

    let expected = ParseError::BadPath;
    let actual = Episode::from(path, &tv_shows(), &allowed_exts()).unwrap_err();

    assert_eq!(actual, expected);
}

#[test]
fn bad_show_name() {
    let path = "/tmp/rusttv-tests/Totally Unknown Show/S01 E01.mkv";
    touch_file(Path::new(path)).unwrap();

    let expected = ParseError::BadShow;
    let actual = Episode::from(path, &tv_shows(), &allowed_exts()).unwrap_err();

    assert_eq!(actual, expected);
}
