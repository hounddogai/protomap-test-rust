use super::*;

use acme_kit::text;

/// Where a fallback verifier target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackVerifierTargetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl FallbackVerifierTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for FallbackVerifierTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback verifier target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackVerifierTarget {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub retries: i64,
    pub cluster: String,
    pub label: String,
    pub status: FallbackVerifierTargetStatus,
}

impl FallbackVerifierTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackVerifierTargetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackVerifierTargetStatus) -> bool {
        match self.status {
            FallbackVerifierTargetStatus::Created => {
                return next == FallbackVerifierTargetStatus::Validated;
            }
            FallbackVerifierTargetStatus::Validated => {
                match next {
                    FallbackVerifierTargetStatus::Published | FallbackVerifierTargetStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackVerifierTargetStatus::Published => {
                match next {
                    FallbackVerifierTargetStatus::Retired => {
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

    pub fn move_to(&mut self, next: FallbackVerifierTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback verifier target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> FallbackVerifierTarget {
        FallbackVerifierTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            retries: self.retries,
            cluster: self.cluster.clone(),
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a fallback verifier target in status created.
pub fn new_fallback_verifier_target(code: &str, name: &str) -> FallbackVerifierTarget {
    FallbackVerifierTarget {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        retries: 0,
        cluster: "".to_string(),
        label: "".to_string(),
        status: FallbackVerifierTargetStatus::Created,
    }
}
