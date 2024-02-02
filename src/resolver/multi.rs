use super::ShowResolver;

pub struct MultiResolver {
    resolvers: Vec<Box<dyn ShowResolver>>,
}

impl MultiResolver {
    pub fn new(resolvers: Vec<Box<dyn ShowResolver>>) -> MultiResolver {
        MultiResolver { resolvers: resolvers }
    }
}

impl ShowResolver for MultiResolver {
    fn resolve(&self, name: &str) -> Option<(String, f64)> {
        for r in &self.resolvers {
            let res = r.resolve(name);
            if res.is_some() {
                return res;
            }
        }
        None
    }
}
