use super::*;

use acme_kit::text;

/// Where a global federation target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFederationTargetStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalFederationTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalFederationTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global federation target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFederationTarget {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub unit: String,
    pub minimum: f64,
    pub maximum: f64,
    pub status: GlobalFederationTargetStatus,
}

impl GlobalFederationTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFederationTargetStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFederationTargetStatus) -> bool {
        match self.status {
            GlobalFederationTargetStatus::Requested => {
                match next {
                    GlobalFederationTargetStatus::Granted | GlobalFederationTargetStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFederationTargetStatus::Granted => {
                return next == GlobalFederationTargetStatus::Expired;
            }
            GlobalFederationTargetStatus::Revoked => {
                return next == GlobalFederationTargetStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalFederationTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global federation target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.minimum + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalFederationTarget {
        GlobalFederationTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            unit: self.unit.clone(),
            minimum: self.minimum,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global federation target in status requested.
pub fn new_global_federation_target(code: &str, name: &str) -> GlobalFederationTarget {
    GlobalFederationTarget {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        unit: "".to_string(),
        minimum: 0.0,
        maximum: 0.0,
        status: GlobalFederationTargetStatus::Requested,
    }
}
