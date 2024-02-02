pub mod multi;
pub mod strsim;
pub mod tmdb;

pub trait ShowResolver {
    // Resolve a show name and return the resolved name and a certainty index (0-1)
    fn resolve(&self, name: &str) -> Option<(String, f64)>;
}
