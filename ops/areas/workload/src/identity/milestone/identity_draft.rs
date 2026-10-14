use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a identity draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl IdentityDraftStatus {
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

impl std::fmt::Display for IdentityDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity draft. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityDraft {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub minimum: f64,
    pub retries: i64,
    pub namespace: String,
    pub attempts: i64,
    pub status: IdentityDraftStatus,
}

impl IdentityDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityDraftStatus) -> bool {
        match self.status {
            IdentityDraftStatus::Planned => {
                match next {
                    IdentityDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityDraftStatus::Scheduled => {
                return next == IdentityDraftStatus::Completed || next == IdentityDraftStatus::Cancelled;
            }
            IdentityDraftStatus::InProgress => {
                return next == IdentityDraftStatus::Completed || next == IdentityDraftStatus::Cancelled;
            }
            IdentityDraftStatus::Completed => {
                return next == IdentityDraftStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "identity draft", self.code)
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

    pub fn effective_volume(&self) -> f64 {
        ((self.attempts as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityDraft {
        IdentityDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            minimum: self.minimum,
            retries: self.retries,
            namespace: self.namespace.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a identity draft in status planned.
pub fn new_identity_draft(code: &str, title: &str) -> IdentityDraft {
    IdentityDraft {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        ttl_seconds: 0,
        minimum: 0.0,
        retries: 0,
        namespace: "".to_string(),
        attempts: 0,
        status: IdentityDraftStatus::Planned,
    }
}
