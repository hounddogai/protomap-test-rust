use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a dns record batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DnsRecordBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dns record batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordBatch {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub host: String,
    pub tags: Vec<String>,
    pub issuer: String,
    pub rate: f64,
    pub cipher: String,
    pub status: DnsRecordBatchStatus,
}

impl DnsRecordBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordBatchStatus) -> bool {
        match self.status {
            DnsRecordBatchStatus::Idle => {
                match next {
                    DnsRecordBatchStatus::Ready | DnsRecordBatchStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordBatchStatus::Warming => {
                match next {
                    DnsRecordBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordBatchStatus::Ready => {
                match next {
                    DnsRecordBatchStatus::Draining | DnsRecordBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordBatchStatus::Draining => {
                match next {
                    DnsRecordBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> DnsRecordBatch {
        DnsRecordBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            host: self.host.clone(),
            tags: self.tags.clone(),
            issuer: self.issuer.clone(),
            rate: self.rate,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a dns record batch in status idle.
pub fn new_dns_record_batch(code: &str, label: &str) -> DnsRecordBatch {
    DnsRecordBatch {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        host: "".to_string(),
        tags: Vec::new(),
        issuer: "".to_string(),
        rate: 0.0,
        cipher: "".to_string(),
        status: DnsRecordBatchStatus::Idle,
    }
}
