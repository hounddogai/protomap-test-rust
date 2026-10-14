use super::*;

use acme_kit::text;

/// Where a batched ttl policy variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTtlPolicyVarianceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedTtlPolicyVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedTtlPolicyVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched ttl policy variance as the ttl policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTtlPolicyVariance {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub cost: f64,
    pub title: String,
    pub pinned: bool,
    pub status: BatchedTtlPolicyVarianceStatus,
}

impl BatchedTtlPolicyVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTtlPolicyVarianceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTtlPolicyVarianceStatus) -> bool {
        match self.status {
            BatchedTtlPolicyVarianceStatus::Created => {
                match next {
                    BatchedTtlPolicyVarianceStatus::Published | BatchedTtlPolicyVarianceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedTtlPolicyVarianceStatus::Validated => {
                return next == BatchedTtlPolicyVarianceStatus::Published
                    || next == BatchedTtlPolicyVarianceStatus::Retired;
            }
            BatchedTtlPolicyVarianceStatus::Published => {
                return next == BatchedTtlPolicyVarianceStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTtlPolicyVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched ttl policy variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> BatchedTtlPolicyVariance {
        BatchedTtlPolicyVariance {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            cost: self.cost,
            title: self.title.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched ttl policy variance in status created.
pub fn new_batched_ttl_policy_variance(code: &str, name: &str) -> BatchedTtlPolicyVariance {
    BatchedTtlPolicyVariance {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        cost: 0.0,
        title: "".to_string(),
        pinned: false,
        status: BatchedTtlPolicyVarianceStatus::Created,
    }
}
