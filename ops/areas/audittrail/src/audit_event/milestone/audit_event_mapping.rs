use super::*;

use acme_kit::text;

/// Where a audit event mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventMappingStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AuditEventMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AuditEventMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One audit event mapping, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventMapping {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub retries: i64,
    pub audience: String,
    pub timezone: String,
    pub status: AuditEventMappingStatus,
}

impl AuditEventMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventMappingStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventMappingStatus) -> bool {
        match self.status {
            AuditEventMappingStatus::Requested => {
                return next == AuditEventMappingStatus::Granted || next == AuditEventMappingStatus::Revoked;
            }
            AuditEventMappingStatus::Granted => {
                match next {
                    AuditEventMappingStatus::Revoked | AuditEventMappingStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventMappingStatus::Revoked => {
                match next {
                    AuditEventMappingStatus::Expired => {
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

    pub fn move_to(&mut self, next: AuditEventMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.retries as f64) * self.threshold * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventMapping {
        AuditEventMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            threshold: self.threshold,
            tags: self.tags.clone(),
            retries: self.retries,
            audience: self.audience.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a audit event mapping in status requested.
pub fn new_audit_event_mapping(code: &str, title: &str) -> AuditEventMapping {
    AuditEventMapping {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        threshold: 0.0,
        tags: Vec::new(),
        retries: 0,
        audience: "".to_string(),
        timezone: "".to_string(),
        status: AuditEventMappingStatus::Requested,
    }
}
