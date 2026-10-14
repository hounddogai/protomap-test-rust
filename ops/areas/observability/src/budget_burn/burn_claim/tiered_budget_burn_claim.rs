use super::*;

use acme_kit::text;

/// Where a tiered budget burn claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredBudgetBurnClaimStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredBudgetBurnClaimStatus {
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

impl std::fmt::Display for TieredBudgetBurnClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered budget burn claim as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredBudgetBurnClaim {
    pub code: String,
    pub title: String,
    pub revision: i64,
    pub source: String,
    pub unit: String,
    pub owner_id: String,
    pub rotation_days: i64,
    pub status: TieredBudgetBurnClaimStatus,
}

impl TieredBudgetBurnClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredBudgetBurnClaimStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredBudgetBurnClaimStatus) -> bool {
        match self.status {
            TieredBudgetBurnClaimStatus::Idle => {
                return next == TieredBudgetBurnClaimStatus::Stopped;
            }
            TieredBudgetBurnClaimStatus::Warming => {
                match next {
                    TieredBudgetBurnClaimStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredBudgetBurnClaimStatus::Ready => {
                match next {
                    TieredBudgetBurnClaimStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredBudgetBurnClaimStatus::Draining => {
                match next {
                    TieredBudgetBurnClaimStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredBudgetBurnClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered budget burn claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered budget burn claim", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.rotation_days as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredBudgetBurnClaim {
        TieredBudgetBurnClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            revision: self.revision,
            source: self.source.clone(),
            unit: self.unit.clone(),
            owner_id: self.owner_id.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a tiered budget burn claim in status idle.
pub fn new_tiered_budget_burn_claim(code: &str, title: &str) -> TieredBudgetBurnClaim {
    TieredBudgetBurnClaim {
        code: code.to_string(),
        title: title.to_string(),
        revision: 0,
        source: "".to_string(),
        unit: "".to_string(),
        owner_id: "".to_string(),
        rotation_days: 0,
        status: TieredBudgetBurnClaimStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_budget_burn_claim("code-476", "title-409");
        assert_eq!(item.status, TieredBudgetBurnClaimStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_tiered_budget_burn_claim("code-476", "title-409");
        if item.move_to(TieredBudgetBurnClaimStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredBudgetBurnClaimStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_budget_burn_claim("code-476", "title-409");
        item.status = TieredBudgetBurnClaimStatus::Stopped;
        assert!(
            item.move_to(TieredBudgetBurnClaimStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
