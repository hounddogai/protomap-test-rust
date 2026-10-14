use super::*;

use acme_kit::text;

/// Where a rollout budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutBudgetStatus {
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

impl std::fmt::Display for RolloutBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutBudget {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub burst: i64,
    pub capacity: i64,
    pub currency: String,
    pub checksum: String,
    pub status: RolloutBudgetStatus,
}

impl RolloutBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutBudgetStatus) -> bool {
        match self.status {
            RolloutBudgetStatus::New => {
                match next {
                    RolloutBudgetStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutBudgetStatus::Queued => {
                return next == RolloutBudgetStatus::Done || next == RolloutBudgetStatus::Failed;
            }
            RolloutBudgetStatus::Running => {
                match next {
                    RolloutBudgetStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutBudgetStatus::Done => {
                return next == RolloutBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.burst as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutBudget {
        RolloutBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            burst: self.burst,
            capacity: self.capacity,
            currency: self.currency.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollout budget in status new.
pub fn new_rollout_budget(code: &str, name: &str) -> RolloutBudget {
    RolloutBudget {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        burst: 0,
        capacity: 0,
        currency: "".to_string(),
        checksum: "".to_string(),
        status: RolloutBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rollout_budget("code-397", "name-450");
        assert_eq!(item.status, RolloutBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_rollout_budget("code-397", "name-450");
        if item.move_to(RolloutBudgetStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RolloutBudgetStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rollout_budget("code-397", "name-450");
        item.status = RolloutBudgetStatus::Failed;
        assert!(item.move_to(RolloutBudgetStatus::New).is_err(), "expected move to fail");
    }
}
