use super::*;

use acme_kit::text;

/// Where a batched budget burn batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBudgetBurnBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedBudgetBurnBatchStatus {
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

impl std::fmt::Display for BatchedBudgetBurnBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched budget burn batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnBatch {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub principal: String,
    pub name: String,
    pub pinned: bool,
    pub status: BatchedBudgetBurnBatchStatus,
}

impl BatchedBudgetBurnBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBudgetBurnBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBudgetBurnBatchStatus) -> bool {
        match self.status {
            BatchedBudgetBurnBatchStatus::New => {
                return next == BatchedBudgetBurnBatchStatus::Queued || next == BatchedBudgetBurnBatchStatus::Done;
            }
            BatchedBudgetBurnBatchStatus::Queued => {
                match next {
                    BatchedBudgetBurnBatchStatus::Done | BatchedBudgetBurnBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBudgetBurnBatchStatus::Running => {
                match next {
                    BatchedBudgetBurnBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBudgetBurnBatchStatus::Done => {
                match next {
                    BatchedBudgetBurnBatchStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedBudgetBurnBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched budget burn batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.maximum
    }

    pub fn copy(&self) -> BatchedBudgetBurnBatch {
        BatchedBudgetBurnBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            principal: self.principal.clone(),
            name: self.name.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched budget burn batch in status new.
pub fn new_batched_budget_burn_batch(code: &str, label: &str) -> BatchedBudgetBurnBatch {
    BatchedBudgetBurnBatch {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        principal: "".to_string(),
        name: "".to_string(),
        pinned: false,
        status: BatchedBudgetBurnBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_budget_burn_batch("code-823", "label-95");
        assert_eq!(item.status, BatchedBudgetBurnBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_batched_budget_burn_batch("code-823", "label-95");
        if item.move_to(BatchedBudgetBurnBatchStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedBudgetBurnBatchStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_budget_burn_batch("code-823", "label-95");
        item.status = BatchedBudgetBurnBatchStatus::Failed;
        assert!(
            item.move_to(BatchedBudgetBurnBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
