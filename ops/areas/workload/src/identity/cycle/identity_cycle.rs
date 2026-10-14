use super::*;

use acme_kit::text;

/// Where a identity cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IdentityCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for IdentityCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityCycle {
    pub code: String,
    pub name: String,
    pub fingerprint: String,
    pub maximum: f64,
    pub quota_units: i64,
    pub tenant_id: String,
    pub status: IdentityCycleStatus,
}

impl IdentityCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityCycleStatus) -> bool {
        match self.status {
            IdentityCycleStatus::Draft => {
                match next {
                    IdentityCycleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityCycleStatus::Pending => {
                return next == IdentityCycleStatus::Suspended;
            }
            IdentityCycleStatus::Active => {
                return next == IdentityCycleStatus::Suspended || next == IdentityCycleStatus::Closed;
            }
            IdentityCycleStatus::Suspended => {
                return next == IdentityCycleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "identity cycle", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.maximum * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityCycle {
        IdentityCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            fingerprint: self.fingerprint.clone(),
            maximum: self.maximum,
            quota_units: self.quota_units,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a identity cycle in status draft.
pub fn new_identity_cycle(code: &str, name: &str) -> IdentityCycle {
    IdentityCycle {
        code: code.to_string(),
        name: name.to_string(),
        fingerprint: "".to_string(),
        maximum: 0.0,
        quota_units: 0,
        tenant_id: "".to_string(),
        status: IdentityCycleStatus::Draft,
    }
}
