use super::*;

use std::path::PathBuf;

use crate::tests as utils;

fn tv_shows() -> Vec<String> {
    vec![
        "The Wild Adventures of the Abyssal Horror",
        "Abyssal Horror-chan Gets a Girlfriend",
        "Snakes are our Friends (2010)",
    ]
    .into_iter()
    .map(String::from)
    .collect()
}

fn allowed_exts() -> Vec<String> {
    vec!["mkv", "mp4"].into_iter().map(String::from).collect()
}

fn reader_skip() -> LocalReader {
    LocalReader::new(tv_shows(), allowed_exts(), FailureAction::Skip)
}

fn reader_abort() -> LocalReader {
    LocalReader::new(tv_shows(), allowed_exts(), FailureAction::Abort)
}

fn setup_all_valid(prefix: &str) -> () {
    let paths = vec![
        "The Wild Adventures of abyssal horror/wild.adventures.abyssal.horror.1x01.1080p.mkv",
        "Snakes are our Friends (2010)/S07 E69.mkv",
    ];

    for p in paths {
        utils::create_path(&format!("{}/{}", prefix, p));
    }
}

fn setup_some_invalid(prefix: &str) -> () {
    setup_all_valid(prefix);
    let paths = vec![
        "sync.sh",
        "The Wild Adventures of abyssal horror/spam file.txt",
        "Totally non-existent show/S01 E01.mkv",
    ];

    for p in paths {
        utils::create_path(&format!("{}/{}", prefix, p));
    }
}

#[test]
fn read_local_skip_invalid_shows() {
    let mut prefix = PathBuf::from(utils::PATH_PREFIX);
    let test_path = "local-skip-invalid";
    prefix.push(test_path);

    setup_some_invalid(test_path);

    let result = reader_skip().read_local(&prefix).unwrap();
    assert_eq!(result.len(), 2);
}

#[test]
fn read_local_abort_on_invalid() {
    let mut prefix = PathBuf::from(utils::PATH_PREFIX);
    let test_path = "local-abort-invalid";
    prefix.push(test_path);

    setup_some_invalid(test_path);

    let expected = ReadError::Aborted;
    let actual = reader_abort().read_local(&prefix).unwrap_err();
    assert_eq!(actual, expected);
}
