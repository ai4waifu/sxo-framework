//! Jupyter wire protocol (multipart ZMQ + HMAC-SHA256).

use bytes::Bytes;
use hmac::{Hmac, Mac};
use serde_json::{Value, json};
use sha2::Sha256;
use uuid::Uuid;
use zeromq::ZmqMessage;

type HmacSha256 = Hmac<Sha256>;

/// Delimiter frame between identities and signed JSON frames.
pub const DELIM: &[u8] = b"<IDS|MSG>";

/// Parsed Jupyter message.
#[derive(Debug, Clone)]
pub struct JupyterMessage {
    /// Routing identities (ROUTER) or IOPub topic frames.
    pub identities: Vec<Bytes>,
    /// Message header JSON.
    pub header: Value,
    /// Parent header JSON (`{}` when absent).
    pub parent_header: Value,
    /// Metadata JSON.
    pub metadata: Value,
    /// Content JSON.
    pub content: Value,
}

impl JupyterMessage {
    /// Message type from the header (`msg_type`).
    pub fn msg_type(&self) -> &str {
        self.header.get("msg_type").and_then(|v| v.as_str()).unwrap_or("")
    }

    /// Session id from the header.
    pub fn session(&self) -> &str {
        self.header.get("session").and_then(|v| v.as_str()).unwrap_or("")
    }

    /// Decode a multipart ZMQ message and verify HMAC when `key` is non-empty.
    pub fn from_zmq(msg: ZmqMessage, key: &str) -> Result<Self, String> {
        let frames = msg.into_vec();
        let delim_pos =
            frames.iter().position(|f| f.as_ref() == DELIM).ok_or_else(|| "missing <IDS|MSG> delimiter".to_string())?;
        if frames.len() < delim_pos + 6 {
            return Err("incomplete Jupyter wire message".into());
        }
        let identities = frames[..delim_pos].to_vec();
        let signature = std::str::from_utf8(&frames[delim_pos + 1]).map_err(|_| "signature is not utf-8".to_string())?;
        let header_bytes = frames[delim_pos + 2].clone();
        let parent_bytes = frames[delim_pos + 3].clone();
        let meta_bytes = frames[delim_pos + 4].clone();
        let content_bytes = frames[delim_pos + 5].clone();

        if !key.is_empty() {
            let expected = sign(key, &[&header_bytes, &parent_bytes, &meta_bytes, &content_bytes]);
            if !constant_time_eq(signature.as_bytes(), expected.as_bytes()) {
                return Err("HMAC signature mismatch".into());
            }
        }

        Ok(Self {
            identities,
            header: parse_json_frame(&header_bytes, "header")?,
            parent_header: parse_json_frame(&parent_bytes, "parent_header")?,
            metadata: parse_json_frame(&meta_bytes, "metadata")?,
            content: parse_json_frame(&content_bytes, "content")?,
        })
    }

    /// Encode this message to a multipart ZMQ message with HMAC.
    pub fn to_zmq(&self, key: &str) -> Result<ZmqMessage, String> {
        let header = serde_json::to_vec(&self.header).map_err(|e| e.to_string())?;
        let parent = serde_json::to_vec(&self.parent_header).map_err(|e| e.to_string())?;
        let metadata = serde_json::to_vec(&self.metadata).map_err(|e| e.to_string())?;
        let content = serde_json::to_vec(&self.content).map_err(|e| e.to_string())?;
        let signature = if key.is_empty() { String::new() } else { sign(key, &[&header, &parent, &metadata, &content]) };

        let mut frames: Vec<Bytes> = self.identities.clone();
        frames.push(Bytes::from_static(DELIM));
        frames.push(Bytes::from(signature));
        frames.push(Bytes::from(header));
        frames.push(Bytes::from(parent));
        frames.push(Bytes::from(metadata));
        frames.push(Bytes::from(content));
        ZmqMessage::try_from(frames).map_err(|_| "empty ZmqMessage".to_string())
    }

    /// Build a reply of `msg_type` with `content`, copying identities and parenting this message.
    pub fn reply(&self, msg_type: &str, content: Value) -> Self {
        Self {
            identities: self.identities.clone(),
            header: make_header(msg_type, self.session(), self.header.get("username")),
            parent_header: self.header.clone(),
            metadata: json!({}),
            content,
        }
    }

    /// Build an IOPub publication parented by this request (topic in identities).
    pub fn iopub(&self, topic: &str, msg_type: &str, content: Value) -> Self {
        Self {
            identities: vec![Bytes::from(topic.to_owned())],
            header: make_header(msg_type, self.session(), self.header.get("username")),
            parent_header: self.header.clone(),
            metadata: json!({}),
            content,
        }
    }
}

/// Create a fresh Jupyter header object.
pub fn make_header(msg_type: &str, session: &str, username: Option<&Value>) -> Value {
    let username = username.and_then(|v| v.as_str()).unwrap_or("sxo").to_string();
    json!({
        "msg_id": Uuid::new_v4().to_string(),
        "session": session,
        "username": username,
        "date": chrono::Utc::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true),
        "msg_type": msg_type,
        "version": "5.3",
    })
}

fn parse_json_frame(bytes: &Bytes, name: &str) -> Result<Value, String> {
    serde_json::from_slice(bytes).map_err(|e| format!("invalid {name} JSON: {e}"))
}

fn sign(key: &str, parts: &[&[u8]]) -> String {
    let mut mac = HmacSha256::new_from_slice(key.as_bytes()).expect("HMAC-SHA256 accepts arbitrary key length");
    for part in parts {
        mac.update(part);
    }
    hex::encode(mac.finalize().into_bytes())
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}
