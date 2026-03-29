use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Request {
    pub method: String,
    pub path: String,
    pub query: HashMap<String, String>,
    pub body: Vec<u8>,
}

impl Request {
    pub(crate) fn from_raw(method: &str, path: &str, query: &str, body: &[u8]) -> Self {
        let query_params: HashMap<String, String> = if query.is_empty() {
            HashMap::new()
        } else {
            query.split('&')
                .filter_map(|pair| {
                    let mut p = pair.splitn(2, '=');
                    Some((
                        urlencoding::decode(p.next()?).unwrap_or_default().into_owned(),
                        urlencoding::decode(p.next().unwrap_or("")).unwrap_or_default().into_owned(),
                    ))
                })
                .collect()
        };

        Self {
            method: method.to_string(),
            path: path.to_string(),
            query: query_params,
            body: body.to_vec(),
        }
    }

    pub fn query(&self, key: &str) -> Option<&str> {
        self.query.get(key).map(|s| s.as_str())
    }

    pub fn json<T: for<'de> Deserialize<'de>>(&self) -> Result<T, serde_json::Error> {
        serde_json::from_slice(&self.body)
    }

    pub fn text(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

#[derive(Debug)]
pub struct Response {
    pub status: u16,
    pub content_type: String,
    pub body: Vec<u8>,
    pub headers: Vec<(String, String)>,
}

impl Response {
    pub fn json<T: Serialize>(data: &T) -> Self {
        Self {
            status: 200,
            content_type: "application/json".to_string(),
            body: serde_json::to_vec(data).unwrap_or_default(),
            headers: vec![],
        }
    }

    pub fn text(text: impl Into<String>) -> Self {
        Self {
            status: 200,
            content_type: "text/plain; charset=utf-8".to_string(),
            body: text.into().into_bytes(),
            headers: vec![],
        }
    }

    pub fn bytes(data: Vec<u8>, content_type: impl Into<String>) -> Self {
        Self {
            status: 200,
            content_type: content_type.into(),
            body: data,
            headers: vec![],
        }
    }

    pub fn error(status: u16, message: impl Into<String>) -> Self {
        let body = serde_json::json!({ "error": message.into() }).to_string();
        Self {
            status,
            content_type: "application/json".to_string(),
            body: body.into_bytes(),
            headers: vec![],
        }
    }

    pub fn with_status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((key.into(), value.into()));
        self
    }
}
