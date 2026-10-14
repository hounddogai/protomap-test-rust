use super::*;

use acme_kit::text;

/// Where a primary limit policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryLimitPolicyStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PrimaryLimitPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PrimaryLimitPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary limit policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryLimitPolicy {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub maximum: f64,
    pub external_ref: String,
    pub region: String,
    pub status: PrimaryLimitPolicyStatus,
}

impl PrimaryLimitPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryLimitPolicyStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryLimitPolicyStatus) -> bool {
        match self.status {
            PrimaryLimitPolicyStatus::Created => {
                return next == PrimaryLimitPolicyStatus::Validated || next == PrimaryLimitPolicyStatus::Published;
            }
            PrimaryLimitPolicyStatus::Validated => {
                return next == PrimaryLimitPolicyStatus::Published;
            }
            PrimaryLimitPolicyStatus::Published => {
                return next == PrimaryLimitPolicyStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryLimitPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary limit policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "primary limit policy", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.attempts as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryLimitPolicy {
        PrimaryLimitPolicy {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            maximum: self.maximum,
            external_ref: self.external_ref.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a primary limit policy in status created.
pub fn new_primary_limit_policy(code: &str, name: &str) -> PrimaryLimitPolicy {
    PrimaryLimitPolicy {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        maximum: 0.0,
        external_ref: "".to_string(),
        region: "".to_string(),
        status: PrimaryLimitPolicyStatus::Created,
    }
}
