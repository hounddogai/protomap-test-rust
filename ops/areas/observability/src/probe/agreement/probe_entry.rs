use super::*;

use acme_kit::text;

/// Where a probe entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ProbeEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ProbeEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeEntry {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub discount: f64,
    pub key_id: String,
    pub minimum: f64,
    pub threshold: f64,
    pub ratio: f64,
    pub status: ProbeEntryStatus,
}

impl ProbeEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeEntryStatus) -> bool {
        match self.status {
            ProbeEntryStatus::Requested => {
                match next {
                    ProbeEntryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeEntryStatus::Granted => {
                match next {
                    ProbeEntryStatus::Revoked | ProbeEntryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeEntryStatus::Revoked => {
                match next {
                    ProbeEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: ProbeEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "probe entry", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ratio - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeEntry {
        ProbeEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            discount: self.discount,
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            threshold: self.threshold,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe entry in status requested.
pub fn new_probe_entry(code: &str, title: &str) -> ProbeEntry {
    ProbeEntry {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        discount: 0.0,
        key_id: "".to_string(),
        minimum: 0.0,
        threshold: 0.0,
        ratio: 0.0,
        status: ProbeEntryStatus::Requested,
    }
}
