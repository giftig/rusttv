use super::*;

use std::env;

#[test]
fn sub_no_replacements() {
    let expected = "no vars here! $$$";
    let actual = sub_vars(&expected);
    assert_eq!(actual, expected);
}

#[test]
fn sub_vars_single_replacement() {
    env::set_var("RUSTTV_TEST1", "my_value");

    let expected = "my_value/bar/baz";
    let actual = sub_vars("${RUSTTV_TEST1}/bar/baz");
    assert_eq!(actual, expected);
}

#[test]
fn sub_vars_multiple_replacements() {
    env::set_var("RUSTTV_TEST2A", "test2a");
    env::set_var("RUSTTV_TEST2B", "test2b");

    let expected = "test2a-test2b";
    let actual = sub_vars("${RUSTTV_TEST2A}-${RUSTTV_TEST2B}");
    assert_eq!(actual, expected);
}
