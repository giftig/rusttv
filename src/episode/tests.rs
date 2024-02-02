use super::*;

use crate::tests as utils;

fn allowed_exts() -> Vec<String> {
    vec!["mkv", "mp4"].into_iter().map(String::from).collect()
}

#[test]
fn valid_episode_exact() {
    let path = utils::test_path("irrelevant.mkv");

    let expected = Episode {
        local_path: path.clone(),
        show_name: String::from("All My Circuits"),
        show_certainty: 1.0,
        season_num: 1,
        episode_num: 2,
        ext: String::from("mkv"),
    };

    let actual = Episode::from(&path, "S01 E02.mkv", "All My Circuits", 1.0, &allowed_exts()).unwrap();

    assert_eq!(actual, expected);
}

#[test]
fn valid_episode_fuzzy() {
    let path = utils::test_path("irrelevant.mkv");

    for f in vec![
        "all.my.circuits.s01e02.1080p.mkv",
        "Calculon Has Amnesia - 1x02.mkv",
        "All.My.Circuits.S01E02.Christmas.Special.1080p.HDTV.H264-FTP[Morbotron.com].mkv",
    ] {
        let expected = Episode {
            local_path: path.clone(),
            show_name: String::from("All My Circuits"),
            show_certainty: 1.0,
            season_num: 1,
            episode_num: 2,
            ext: String::from("mkv"),
        };

        let actual = Episode::from(&path, f, "All My Circuits", 1.0, &allowed_exts()).unwrap();

        assert_eq!(actual, expected);
    }
}
