use base64::{engine::general_purpose, Engine as _};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EncodingType {
    Base64,
    Url,
    Hex,
    Html,
    Rot13,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformRequest {
    pub text: String,
    pub encoding: EncodingType,
    pub operation: TransformOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransformOperation {
    Encode,
    Decode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformResponse {
    pub text: String,
    pub error: Option<String>,
}

pub struct Encoder;

impl Encoder {
    pub fn transform(req: TransformRequest) -> TransformResponse {
        match req.operation {
            TransformOperation::Encode => Self::encode(req.text, req.encoding),
            TransformOperation::Decode => Self::decode(req.text, req.encoding),
        }
    }

    fn encode(input: String, encoding: EncodingType) -> TransformResponse {
        let result = match encoding {
            EncodingType::Base64 => Ok(general_purpose::STANDARD.encode(input)),
            EncodingType::Url => Ok(urlencoding::encode(&input).to_string()),
            EncodingType::Hex => Ok(hex::encode(input)),
            EncodingType::Html => Ok(html_escape::encode_text(&input).to_string()),
            EncodingType::Rot13 => Ok(Self::rot13(&input)),
        };

        match result {
            Ok(text) => TransformResponse { text, error: None },
            Err(e) => TransformResponse {
                text: String::new(),
                error: Some(e.to_string()),
            },
        }
    }

    fn decode(input: String, encoding: EncodingType) -> TransformResponse {
        let result = match encoding {
            EncodingType::Base64 => general_purpose::STANDARD
                .decode(&input)
                .map_err(|e| e.to_string())
                .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string())),
            EncodingType::Url => urlencoding::decode(&input)
                .map(|s| s.to_string())
                .map_err(|e| e.to_string()),
            EncodingType::Hex => hex::decode(&input)
                .map_err(|e| e.to_string())
                .and_then(|bytes| String::from_utf8(bytes).map_err(|e| e.to_string())),
            EncodingType::Html => Ok(html_escape::decode_html_entities(&input).to_string()),
            EncodingType::Rot13 => Ok(Self::rot13(&input)),
        };

        match result {
            Ok(text) => TransformResponse { text, error: None },
            Err(e) => TransformResponse {
                text: String::new(),
                error: Some(e),
            },
        }
    }

    fn rot13(input: &str) -> String {
        input
            .chars()
            .map(|c| match c {
                'a'..='m' | 'A'..='M' => ((c as u8) + 13) as char,
                'n'..='z' | 'N'..='Z' => ((c as u8) - 13) as char,
                _ => c,
            })
            .collect()
    }
}
