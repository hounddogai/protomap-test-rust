use super::*;

use acme_kit::text;

/// Where a budget burn batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BudgetBurnBatchStatus {
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

impl std::fmt::Display for BudgetBurnBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn batch and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnBatch {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub contact_email: String,
    pub segment: String,
    pub margin: f64,
    pub status: BudgetBurnBatchStatus,
}

impl BudgetBurnBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnBatchStatus) -> bool {
        match self.status {
            BudgetBurnBatchStatus::New => {
                match next {
                    BudgetBurnBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnBatchStatus::Queued => {
                match next {
                    BudgetBurnBatchStatus::Done | BudgetBurnBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnBatchStatus::Running => {
                return next == BudgetBurnBatchStatus::Done;
            }
            BudgetBurnBatchStatus::Done => {
                return next == BudgetBurnBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "budget burn batch", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> BudgetBurnBatch {
        BudgetBurnBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            contact_email: self.contact_email.clone(),
            segment: self.segment.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a budget burn batch in status new.
pub fn new_budget_burn_batch(code: &str, label: &str) -> BudgetBurnBatch {
    BudgetBurnBatch {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        contact_email: "".to_string(),
        segment: "".to_string(),
        margin: 0.0,
        status: BudgetBurnBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_budget_burn_batch("code-146", "label-582");
        assert_eq!(item.status, BudgetBurnBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_budget_burn_batch("code-146", "label-582");
        if item.move_to(BudgetBurnBatchStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnBatchStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_budget_burn_batch("code-146", "label-582");
        item.status = BudgetBurnBatchStatus::Failed;
        assert!(
            item.move_to(BudgetBurnBatchStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
