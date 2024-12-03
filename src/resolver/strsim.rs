#[cfg(test)]
mod tests;

use strsim;

use super::ShowResolver;

const SIM_THRESHOLD_PERFECT: f64 = 0.9;
const SIM_THRESHOLD_GOOD: f64 = 0.75;

const SHORT_LENGTH: usize = 10;
const SIM_THRESHOLD_GOOD_SHORT: f64 = 0.85;  // with very short inputs

pub struct StrsimResolver {
    known_shows: Vec<String>,
}

impl StrsimResolver {
    pub fn new<T: AsRef<str>>(known_shows: &[T]) -> StrsimResolver {
        let ks = known_shows.iter().map(|s| s.as_ref().to_string()).collect();
        StrsimResolver { known_shows: ks }
    }
}

impl ShowResolver for StrsimResolver {
    fn resolve(&self, name: &str) -> Option<(String, f64)> {
        if self.known_shows.contains(&name.to_string()) {
            return Some((name.to_string(), 1.0));
        }

        let mut best_thresh: f64 = 0.0;
        let mut best_match: Option<String> = None;

        for known in &self.known_shows {
            let thresh = strsim::jaro(name, &known);

            if thresh >= SIM_THRESHOLD_PERFECT {
                return Some((known.clone(), thresh));
            }

            if thresh > best_thresh {
                best_thresh = thresh;
                best_match = Some(known.clone());
            }
        }

        if let Some(m) = best_match {
            if best_thresh >= SIM_THRESHOLD_GOOD_SHORT {
                return Some((m, best_thresh));
            }

            if m.len() > SHORT_LENGTH && best_thresh >= SIM_THRESHOLD_GOOD {
                return Some((m, best_thresh));
            }
        }

        None
    }
}
