use regex::Regex;
use serde_json::Value as JsValue;
use ureq;

use super::ShowResolver;

pub struct TmdbResolver {
    protocol: String,
    host: String,
    token: String,
}

impl TmdbResolver {
    pub fn new(protocol: &str, host: &str, token: &str) -> TmdbResolver {
        TmdbResolver {
            protocol: protocol.to_string(),
            host: host.to_string(),
            token: token.to_string(),
        }
    }

    // TMDB api sucks and won't find results if you add a year to the end, even
    // if that year is correct!
    fn strip_year(name: &str) -> String {
        let pattern = Regex::new(r"\([0-9]+\)$").unwrap();
        let res = pattern.replace(name, "").into_owned();

        res
    }

    // It's possible for the name to contain special characters which will break
    // path formatting; make sure problem characters are replaced with a space
    fn sanitise_name(name: &str) -> String {
        name.replace("/", " ").replace("\\", " ")
    }

    fn get_first_match(&self, name: &str) -> Option<String> {
        let url = format!("{}://{}/3/search/tv", self.protocol, self.host);
        let req = ureq::get(&url)
            .set("Authorization", &format!("Bearer {}", self.token))
            .query("query", &Self::strip_year(name));

        let res = req.call().ok()?;

        res.into_json::<JsValue>()
            .ok()?
            .get("results")?
            .as_array()?
            .first()?
            .get("name")?
            .as_str()
            .map(Self::sanitise_name)
    }
}

impl ShowResolver for TmdbResolver {
    // Return 10% certainty as we really need to ask user to confirm, here
    fn resolve(&self, name: &str) -> Option<(String, f64)> {
        self.get_first_match(name).map(|n| (n, 0.1))
    }
}
