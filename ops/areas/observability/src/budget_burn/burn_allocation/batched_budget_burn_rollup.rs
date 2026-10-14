use super::*;

use acme_kit::text;

/// Where a batched budget burn rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBudgetBurnRollupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedBudgetBurnRollupStatus {
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

impl std::fmt::Display for BatchedBudgetBurnRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched budget burn rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnRollup {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub archived: bool,
    pub namespace: String,
    pub ratio: f64,
    pub revision: i64,
    pub status: BatchedBudgetBurnRollupStatus,
}

impl BatchedBudgetBurnRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBudgetBurnRollupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBudgetBurnRollupStatus) -> bool {
        match self.status {
            BatchedBudgetBurnRollupStatus::Open => {
                match next {
                    BatchedBudgetBurnRollupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBudgetBurnRollupStatus::InReview => {
                return next == BatchedBudgetBurnRollupStatus::Resolved
                    || next == BatchedBudgetBurnRollupStatus::Archived;
            }
            BatchedBudgetBurnRollupStatus::Resolved => {
                return next == BatchedBudgetBurnRollupStatus::Reopened;
            }
            BatchedBudgetBurnRollupStatus::Reopened => {
                return next == BatchedBudgetBurnRollupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedBudgetBurnRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched budget burn rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.revision as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedBudgetBurnRollup {
        BatchedBudgetBurnRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            archived: self.archived,
            namespace: self.namespace.clone(),
            ratio: self.ratio,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched budget burn rollup in status open.
pub fn new_batched_budget_burn_rollup(code: &str, name: &str) -> BatchedBudgetBurnRollup {
    BatchedBudgetBurnRollup {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        archived: false,
        namespace: "".to_string(),
        ratio: 0.0,
        revision: 0,
        status: BatchedBudgetBurnRollupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_budget_burn_rollup("code-38", "name-711");
        assert_eq!(item.status, BatchedBudgetBurnRollupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_batched_budget_burn_rollup("code-38", "name-711");
        if item.move_to(BatchedBudgetBurnRollupStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedBudgetBurnRollupStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_budget_burn_rollup("code-38", "name-711");
        item.status = BatchedBudgetBurnRollupStatus::Archived;
        assert!(
            item.move_to(BatchedBudgetBurnRollupStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
