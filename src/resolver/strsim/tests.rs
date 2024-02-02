use super::*;

fn get_resolver() -> StrsimResolver {
    let known_shows = vec![
        "All My Circuits",
        "Everybody Loves Hypnotoad",
        "Calculon (2010)",
        "Calculon: A Calculon Story",
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

        assert_eq!(actual, show);
        assert_eq!(certainty, 1.0);
    }
}

#[test]
fn resolve_fuzzy() {
    let resolver = get_resolver();

    for (fuzzy, expected) in vec![
        ("all my circuits", "All My Circuits"),
        ("All My Circuits (2011)", "All My Circuits"),
        ("calculon", "Calculon (2010)"),
        ("Calculon a Calculon Story", "Calculon: A Calculon Story"),
        (
            "calculon a calculon story (2011)",
            "Calculon: A Calculon Story",
        ),
    ] {
        let (actual, _) = resolver.resolve(fuzzy).unwrap();
        assert_eq!(actual, expected);
    }
}
