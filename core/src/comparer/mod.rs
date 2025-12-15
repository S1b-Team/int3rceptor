use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareRequest {
    pub left: String,
    pub right: String,
    pub mode: CompareMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CompareMode {
    Lines,
    Words,
    Chars,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffChange {
    pub tag: String, // "equal", "delete", "insert"
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompareResponse {
    pub changes: Vec<DiffChange>,
}

pub struct Comparer;

impl Comparer {
    pub fn compare(req: CompareRequest) -> CompareResponse {
        let diff = match req.mode {
            CompareMode::Lines => TextDiff::from_lines(&req.left, &req.right),
            CompareMode::Words => TextDiff::from_words(&req.left, &req.right),
            CompareMode::Chars => TextDiff::from_chars(&req.left, &req.right),
        };

        let mut changes = Vec::new();

        for change in diff.iter_all_changes() {
            let tag = match change.tag() {
                ChangeTag::Equal => "equal",
                ChangeTag::Delete => "delete",
                ChangeTag::Insert => "insert",
            };

            changes.push(DiffChange {
                tag: tag.to_string(),
                value: change.value().to_string(),
            });
        }

        CompareResponse { changes }
    }
}
