use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a probe notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeNoticeStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ProbeNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for ProbeNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A probe notice as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeNotice {
    pub code: String,
    pub label: String,
    pub expires_at: i64,
    pub threshold: f64,
    pub starts_at: i64,
    pub external_ref: String,
    pub revision: i64,
    pub attributes: BTreeMap<String, String>,
    pub floor_value: f64,
    pub port: i64,
    pub status: ProbeNoticeStatus,
}

impl ProbeNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeNoticeStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeNoticeStatus) -> bool {
        match self.status {
            ProbeNoticeStatus::Planned => {
                match next {
                    ProbeNoticeStatus::Scheduled | ProbeNoticeStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeNoticeStatus::Scheduled => {
                match next {
                    ProbeNoticeStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeNoticeStatus::InProgress => {
                return next == ProbeNoticeStatus::Cancelled;
            }
            ProbeNoticeStatus::Completed => {
                return next == ProbeNoticeStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ProbeNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn effective_total(&self) -> f64 {
        ((self.revision as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ProbeNotice {
        ProbeNotice {
            code: self.code.clone(),
            label: self.label.clone(),
            expires_at: self.expires_at,
            threshold: self.threshold,
            starts_at: self.starts_at,
            external_ref: self.external_ref.clone(),
            revision: self.revision,
            attributes: self.attributes.clone(),
            floor_value: self.floor_value,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a probe notice in status planned.
pub fn new_probe_notice(code: &str, label: &str) -> ProbeNotice {
    ProbeNotice {
        code: code.to_string(),
        label: label.to_string(),
        expires_at: 0,
        threshold: 0.0,
        starts_at: 0,
        external_ref: "".to_string(),
        revision: 0,
        attributes: BTreeMap::new(),
        floor_value: 0.0,
        port: 0,
        status: ProbeNoticeStatus::Planned,
    }
}
