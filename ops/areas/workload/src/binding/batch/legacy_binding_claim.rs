use super::*;

use acme_kit::text;

/// Where a legacy binding claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyBindingClaimStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LegacyBindingClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LegacyBindingClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy binding claim as the binding team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyBindingClaim {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub grace_days: i64,
    pub ceiling: f64,
    pub title: String,
    pub reviewer: String,
    pub status: LegacyBindingClaimStatus,
}

impl LegacyBindingClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyBindingClaimStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyBindingClaimStatus) -> bool {
        match self.status {
            LegacyBindingClaimStatus::Created => {
                match next {
                    LegacyBindingClaimStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyBindingClaimStatus::Validated => {
                return next == LegacyBindingClaimStatus::Published || next == LegacyBindingClaimStatus::Retired;
            }
            LegacyBindingClaimStatus::Published => {
                return next == LegacyBindingClaimStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyBindingClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy binding claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ceiling + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyBindingClaim {
        LegacyBindingClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy binding claim in status created.
pub fn new_legacy_binding_claim(code: &str, name: &str) -> LegacyBindingClaim {
    LegacyBindingClaim {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        grace_days: 0,
        ceiling: 0.0,
        title: "".to_string(),
        reviewer: "".to_string(),
        status: LegacyBindingClaimStatus::Created,
    }
}
