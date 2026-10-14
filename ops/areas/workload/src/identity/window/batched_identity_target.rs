use super::*;

use acme_kit::text;

/// Where a batched identity target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedIdentityTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedIdentityTargetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedIdentityTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched identity target. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedIdentityTarget {
    pub code: String,
    pub name: String,
    pub category: String,
    pub budget: f64,
    pub cost: f64,
    pub sequence: i64,
    pub fingerprint: String,
    pub external_ref: String,
    pub currency: String,
    pub status: BatchedIdentityTargetStatus,
}

impl BatchedIdentityTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedIdentityTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedIdentityTargetStatus) -> bool {
        match self.status {
            BatchedIdentityTargetStatus::Idle => {
                match next {
                    BatchedIdentityTargetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedIdentityTargetStatus::Warming => {
                return next == BatchedIdentityTargetStatus::Ready || next == BatchedIdentityTargetStatus::Stopped;
            }
            BatchedIdentityTargetStatus::Ready => {
                match next {
                    BatchedIdentityTargetStatus::Draining | BatchedIdentityTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedIdentityTargetStatus::Draining => {
                match next {
                    BatchedIdentityTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: BatchedIdentityTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched identity target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched identity target", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        (self.budget * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedIdentityTarget {
        BatchedIdentityTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            category: self.category.clone(),
            budget: self.budget,
            cost: self.cost,
            sequence: self.sequence,
            fingerprint: self.fingerprint.clone(),
            external_ref: self.external_ref.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched identity target in status idle.
pub fn new_batched_identity_target(code: &str, name: &str) -> BatchedIdentityTarget {
    BatchedIdentityTarget {
        code: code.to_string(),
        name: name.to_string(),
        category: "".to_string(),
        budget: 0.0,
        cost: 0.0,
        sequence: 0,
        fingerprint: "".to_string(),
        external_ref: "".to_string(),
        currency: "".to_string(),
        status: BatchedIdentityTargetStatus::Idle,
    }
}
