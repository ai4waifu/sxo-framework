//! Jupyter connection-file parsing.

use serde::Deserialize;

/// Ports and auth material from a Jupyter kernel connection JSON file.
#[derive(Debug, Clone, Deserialize)]
pub struct ConnectionFile {
    /// Shell (ROUTER) port.
    pub shell_port: u16,
    /// IOPub (PUB) port.
    pub iopub_port: u16,
    /// Stdin (ROUTER) port.
    pub stdin_port: u16,
    /// Control (ROUTER) port.
    pub control_port: u16,
    /// Heartbeat (REP) port.
    pub hb_port: u16,
    /// Bind address (usually `127.0.0.1`).
    pub ip: String,
    /// HMAC key (may be empty when signing is disabled).
    pub key: String,
    /// Transport scheme (`tcp`).
    #[serde(default = "default_transport")]
    pub transport: String,
    /// Signature scheme (`hmac-sha256`).
    #[serde(default = "default_signature_scheme")]
    pub signature_scheme: String,
}

fn default_transport() -> String {
    "tcp".to_string()
}

fn default_signature_scheme() -> String {
    "hmac-sha256".to_string()
}

impl ConnectionFile {
    /// Load and parse a connection file from disk.
    pub fn load(path: &str) -> Result<Self, String> {
        let raw = std::fs::read_to_string(path).map_err(|e| format!("failed to read connection file {path}: {e}"))?;
        let conn: Self = serde_json::from_str(&raw).map_err(|e| format!("invalid connection file JSON: {e}"))?;
        if conn.transport != "tcp" {
            return Err(format!("unsupported transport: {}", conn.transport));
        }
        if !conn.signature_scheme.is_empty()
            && conn.signature_scheme != "hmac-sha256"
            && conn.signature_scheme != "hmac-sha256.digest"
        {
            return Err(format!("unsupported signature_scheme: {}", conn.signature_scheme));
        }
        Ok(conn)
    }

    /// Build a ZeroMQ endpoint for the given port.
    pub fn endpoint(&self, port: u16) -> String {
        format!("{}://{}:{}", self.transport, self.ip, port)
    }
}
