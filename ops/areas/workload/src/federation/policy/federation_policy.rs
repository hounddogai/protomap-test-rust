use super::*;

use acme_kit::text;

/// Where a federation policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationPolicyStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FederationPolicyStatus {
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

impl std::fmt::Display for FederationPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationPolicy {
    pub code: String,
    pub label: String,
    pub region: String,
    pub country: String,
    pub score: f64,
    pub status: FederationPolicyStatus,
}

impl FederationPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationPolicyStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationPolicyStatus) -> bool {
        match self.status {
            FederationPolicyStatus::Draft => {
                match next {
                    FederationPolicyStatus::Active | FederationPolicyStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationPolicyStatus::Pending => {
                return next == FederationPolicyStatus::Active;
            }
            FederationPolicyStatus::Active => {
                return next == FederationPolicyStatus::Suspended || next == FederationPolicyStatus::Closed;
            }
            FederationPolicyStatus::Suspended => {
                return next == FederationPolicyStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> FederationPolicy {
        FederationPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            country: self.country.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a federation policy in status draft.
pub fn new_federation_policy(code: &str, label: &str) -> FederationPolicy {
    FederationPolicy {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        country: "".to_string(),
        score: 0.0,
        status: FederationPolicyStatus::Draft,
    }
}
