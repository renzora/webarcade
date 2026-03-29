use crate::request::{Request, Response};
use std::collections::HashMap;
use std::sync::Arc;

pub type HandlerFn = Arc<dyn Fn(Request) -> Response + Send + Sync>;

pub struct Router {
    routes: HashMap<String, HandlerFn>,
}

impl Router {
    pub fn new() -> Self {
        Self { routes: HashMap::new() }
    }

    pub fn add<F>(&mut self, method: &str, path: &str, handler: F)
    where
        F: Fn(Request) -> Response + Send + Sync + 'static,
    {
        let key = make_key(method, path);
        self.routes.insert(key, Arc::new(handler));
    }

    pub fn match_route(&self, method: &str, path: &str) -> Option<&HandlerFn> {
        let key = make_key(method, path);
        self.routes.get(&key)
    }
}

fn make_key(method: &str, path: &str) -> String {
    let mut key = String::with_capacity(method.len() + 1 + path.len());
    for c in method.chars() {
        key.push(c.to_ascii_uppercase());
    }
    key.push(' ');
    key.push_str(path);
    key
}
