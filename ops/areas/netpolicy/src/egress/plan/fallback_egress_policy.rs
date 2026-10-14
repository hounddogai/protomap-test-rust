use super::*;

use acme_kit::text;

/// Where a fallback egress policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEgressPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackEgressPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for FallbackEgressPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback egress policy. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEgressPolicy {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub currency: String,
    pub external_ref: String,
    pub version: i64,
    pub segment: String,
    pub status: FallbackEgressPolicyStatus,
}

impl FallbackEgressPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEgressPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEgressPolicyStatus) -> bool {
        match self.status {
            FallbackEgressPolicyStatus::Open => {
                return next == FallbackEgressPolicyStatus::Reopened || next == FallbackEgressPolicyStatus::Archived;
            }
            FallbackEgressPolicyStatus::InReview => {
                match next {
                    FallbackEgressPolicyStatus::Resolved | FallbackEgressPolicyStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressPolicyStatus::Resolved => {
                match next {
                    FallbackEgressPolicyStatus::Reopened | FallbackEgressPolicyStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEgressPolicyStatus::Reopened => {
                match next {
                    FallbackEgressPolicyStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackEgressPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback egress policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.version as f64
    }

    pub fn copy(&self) -> FallbackEgressPolicy {
        FallbackEgressPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            currency: self.currency.clone(),
            external_ref: self.external_ref.clone(),
            version: self.version,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback egress policy in status open.
pub fn new_fallback_egress_policy(code: &str, label: &str) -> FallbackEgressPolicy {
    FallbackEgressPolicy {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        currency: "".to_string(),
        external_ref: "".to_string(),
        version: 0,
        segment: "".to_string(),
        status: FallbackEgressPolicyStatus::Open,
    }
}
