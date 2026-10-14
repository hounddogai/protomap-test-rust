use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary audit event request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryAuditEventRequestStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryAuditEventRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryAuditEventRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary audit event request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryAuditEventRequest {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub window_days: i64,
    pub expires_at: i64,
    pub status: PrimaryAuditEventRequestStatus,
}

impl PrimaryAuditEventRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryAuditEventRequestStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryAuditEventRequestStatus) -> bool {
        match self.status {
            PrimaryAuditEventRequestStatus::Requested => {
                match next {
                    PrimaryAuditEventRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryAuditEventRequestStatus::Granted => {
                match next {
                    PrimaryAuditEventRequestStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryAuditEventRequestStatus::Revoked => {
                match next {
                    PrimaryAuditEventRequestStatus::Expired => {
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

    pub fn move_to(&mut self, next: PrimaryAuditEventRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary audit event request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary audit event request", self.code)
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

    pub fn adjusted_volume(&self) -> f64 {
        ((self.expires_at as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryAuditEventRequest {
        PrimaryAuditEventRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            window_days: self.window_days,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary audit event request in status requested.
pub fn new_primary_audit_event_request(code: &str, name: &str) -> PrimaryAuditEventRequest {
    PrimaryAuditEventRequest {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        window_days: 0,
        expires_at: 0,
        status: PrimaryAuditEventRequestStatus::Requested,
    }
}
