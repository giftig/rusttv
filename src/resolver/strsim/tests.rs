use super::*;

fn get_resolver() -> StrsimResolver {
    let known_shows = vec![
        "All My Circuits",
        "Everybody Loves Hypnotoad",
        "Calculon (2010)",
        "Calculon: A Calculon Story",
        "The Never Ending Story",
        "The Chase",
        "The Bar",
        "The Run",
    ];
    StrsimResolver::new(&known_shows)
}

#[test]
fn resolve_exact() {
    let resolver = get_resolver();

    for show in vec![
        "All My Circuits",
        "Everybody Loves Hypnotoad",
        "Calculon (2010)",
        "Calculon: A Calculon Story",
    ] {
        let (actual, certainty) = resolver.resolve(show).unwrap();

        assert_eq!(actual, show, "Input: {}", show);
        assert_eq!(certainty, 1.0, "Exact match. Input: {}", show);
    }
}

#[test]
fn resolve_fuzzy() {
    let resolver = get_resolver();

    for (fuzzy, expected) in vec![
        ("all my circuits", "All My Circuits"),
        ("All My Circuits (2011)", "All My Circuits"),
        ("Calculon a Calculon Story", "Calculon: A Calculon Story"),
        (
            "calculon a calculon story (2011)",
            "Calculon: A Calculon Story",
        ),
        ("Never ending story", "The Never Ending Story"),
        ("The Chsae", "The Chase"),
    ] {
        let (actual, _) = resolver.resolve(fuzzy).expect(&format!("Match expected for {}", fuzzy));
        assert_eq!(actual, expected, "Input: {}", fuzzy);
    }
}

#[test]
fn no_resolve() {
    let resolver = get_resolver();

    for show in vec!["The March", "The Foo", "The Fuzz"] {
        let res = resolver.resolve(show);
        assert_eq!(res, None, "Input: {}", show);
    }
}
