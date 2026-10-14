use super::*;

use acme_kit::text;

/// Where a strict audit event profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictAuditEventProfileStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl StrictAuditEventProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for StrictAuditEventProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a strict audit event profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictAuditEventProfile {
    pub code: String,
    pub name: String,
    pub description: String,
    pub tags: Vec<String>,
    pub channel: String,
    pub quantity: i64,
    pub status: StrictAuditEventProfileStatus,
}

impl StrictAuditEventProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictAuditEventProfileStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictAuditEventProfileStatus) -> bool {
        match self.status {
            StrictAuditEventProfileStatus::Created => {
                match next {
                    StrictAuditEventProfileStatus::Validated | StrictAuditEventProfileStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictAuditEventProfileStatus::Validated => {
                return next == StrictAuditEventProfileStatus::Published;
            }
            StrictAuditEventProfileStatus::Published => {
                match next {
                    StrictAuditEventProfileStatus::Retired => {
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

    pub fn move_to(&mut self, next: StrictAuditEventProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict audit event profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> StrictAuditEventProfile {
        StrictAuditEventProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            tags: self.tags.clone(),
            channel: self.channel.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a strict audit event profile in status created.
pub fn new_strict_audit_event_profile(code: &str, name: &str) -> StrictAuditEventProfile {
    StrictAuditEventProfile {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        tags: Vec::new(),
        channel: "".to_string(),
        quantity: 0,
        status: StrictAuditEventProfileStatus::Created,
    }
}
