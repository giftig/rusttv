use super::*;

use std::path::{Path, PathBuf};

use claim::*;

use crate::tests as utils;

fn tv_shows() -> Vec<String> {
    vec![
        "All My Circuits",
        "Everybody Loves Hypnotoad",
        "Calculon (2010)",
        "Calculon: A Calculon Story",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn allowed_exts() -> Vec<String> {
    vec!["mkv", "mp4"].into_iter().map(String::from).collect()
}

#[test]
fn valid_episode_exact() {
    let path = utils::create_path("All My Circuits/S01 E02.mkv");

    let expected = Episode {
        local_path: path.clone(),
        show_name: String::from("All My Circuits"),
        show_certainty: 1.0,
        season_num: 1,
        episode_num: 2,
        ext: String::from("mkv"),
    };

    let actual = Episode::from(&path, &tv_shows(), &allowed_exts()).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn valid_episode_fuzzy() {
    let prefix = utils::test_path("All My Circuits/");

    for f in vec![
        "all.my.circuits.s01e02.1080p.mkv",
        "Calculon Has Amnesia - 1x02.mkv",
        "All.My.Circuits.S01E02.Christmas.Special.1080p.HDTV.H264-FTP[Morbotron.com].mkv",
    ] {
        let mut path = prefix.clone();
        path.push(f);

        utils::touch_file(&path).unwrap();

        let expected = Episode {
            local_path: path.clone(),
            show_name: String::from("All My Circuits"),
            show_certainty: 1.0,
            season_num: 1,
            episode_num: 2,
            ext: String::from("mkv"),
        };

        let actual = Episode::from(&path, &tv_shows(), &allowed_exts()).unwrap();

        assert_eq!(actual, expected);
    }
}

#[test]
fn valid_show_fuzzy() {
    let prefix = PathBuf::from(utils::PATH_PREFIX);

    for (fuzzy, exact) in vec![
        ("all my circuits", "All My Circuits"),
        ("All My Circuits (2011)", "All My Circuits"),
        ("calculon", "Calculon (2010)"),
        ("Calculon a Calculon Story", "Calculon: A Calculon Story"),
        (
            "calculon a calculon story (2011)",
            "Calculon: A Calculon Story",
        ),
    ] {
        let mut path = prefix.clone();
        path.push(fuzzy);
        path.push("S00 E00.mp4");
        utils::touch_file(&path).unwrap();

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
    let path = Path::new("/");

    let expected = ParseError::BadPath;
    let actual = Episode::from(path, &tv_shows(), &allowed_exts()).unwrap_err();

    assert_eq!(actual, expected);
}

#[test]
fn bad_show_name() {
    let path = utils::create_path("Totally Unknown Show/S01 E01.mkv");

    let expected = ParseError::BadShow;
    let actual = Episode::from(&path, &tv_shows(), &allowed_exts()).unwrap_err();

    assert_eq!(actual, expected);
}
