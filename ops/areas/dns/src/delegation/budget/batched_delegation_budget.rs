use super::*;

use acme_kit::text;

/// Where a batched delegation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedDelegationBudgetStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedDelegationBudgetStatus {
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

impl std::fmt::Display for BatchedDelegationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched delegation budget as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDelegationBudget {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub category: String,
    pub discount: f64,
    pub reviewer: String,
    pub status: BatchedDelegationBudgetStatus,
}

impl BatchedDelegationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedDelegationBudgetStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedDelegationBudgetStatus) -> bool {
        match self.status {
            BatchedDelegationBudgetStatus::New => {
                return next == BatchedDelegationBudgetStatus::Queued;
            }
            BatchedDelegationBudgetStatus::Queued => {
                return next == BatchedDelegationBudgetStatus::Failed;
            }
            BatchedDelegationBudgetStatus::Running => {
                match next {
                    BatchedDelegationBudgetStatus::Done | BatchedDelegationBudgetStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDelegationBudgetStatus::Done => {
                return next == BatchedDelegationBudgetStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedDelegationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched delegation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> BatchedDelegationBudget {
        BatchedDelegationBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            category: self.category.clone(),
            discount: self.discount,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched delegation budget in status new.
pub fn new_batched_delegation_budget(code: &str, name: &str) -> BatchedDelegationBudget {
    BatchedDelegationBudget {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        category: "".to_string(),
        discount: 0.0,
        reviewer: "".to_string(),
        status: BatchedDelegationBudgetStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_delegation_budget("code-210", "name-607");
        assert_eq!(item.status, BatchedDelegationBudgetStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_delegation_budget("code-210", "name-607");
        if item.move_to(BatchedDelegationBudgetStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedDelegationBudgetStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_delegation_budget("code-210", "name-607");
        item.status = BatchedDelegationBudgetStatus::Failed;
        assert!(
            item.move_to(BatchedDelegationBudgetStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
