use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HeaderPatch {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RepeatRequest {
    pub id: u64,
    pub method: Option<String>,
    pub url: Option<String>,
    pub headers: Option<Vec<HeaderPatch>>,
    pub modified_body: Option<String>,
}
