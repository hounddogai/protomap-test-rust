use super::*;

use acme_kit::text;

/// Where a tiered delegation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDelegationClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredDelegationClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for TieredDelegationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered delegation claim as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDelegationClaim {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub rotation_days: i64,
    pub cost: f64,
    pub batch_size: i64,
    pub status: TieredDelegationClaimStatus,
}

impl TieredDelegationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDelegationClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDelegationClaimStatus) -> bool {
        match self.status {
            TieredDelegationClaimStatus::New => {
                match next {
                    TieredDelegationClaimStatus::Running | TieredDelegationClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDelegationClaimStatus::Queued => {
                match next {
                    TieredDelegationClaimStatus::Running | TieredDelegationClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDelegationClaimStatus::Running => {
                match next {
                    TieredDelegationClaimStatus::Done | TieredDelegationClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDelegationClaimStatus::Done => {
                return next == TieredDelegationClaimStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredDelegationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered delegation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "tiered delegation claim", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rotation_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDelegationClaim {
        TieredDelegationClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            cost: self.cost,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a tiered delegation claim in status new.
pub fn new_tiered_delegation_claim(code: &str, label: &str) -> TieredDelegationClaim {
    TieredDelegationClaim {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        rotation_days: 0,
        cost: 0.0,
        batch_size: 0,
        status: TieredDelegationClaimStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_tiered_delegation_claim("code-490", "label-325");
        assert_eq!(item.status, TieredDelegationClaimStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_tiered_delegation_claim("code-490", "label-325");
        if item.move_to(TieredDelegationClaimStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDelegationClaimStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_tiered_delegation_claim("code-490", "label-325");
        item.status = TieredDelegationClaimStatus::Failed;
        assert!(
            item.move_to(TieredDelegationClaimStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
