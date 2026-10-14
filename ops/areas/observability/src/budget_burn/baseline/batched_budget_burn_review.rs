use super::*;

use acme_kit::text;

/// Where a batched budget burn review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedBudgetBurnReviewStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedBudgetBurnReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedBudgetBurnReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched budget burn review as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedBudgetBurnReview {
    pub code: String,
    pub name: String,
    pub country: String,
    pub priority: i64,
    pub approved: bool,
    pub category: String,
    pub status: BatchedBudgetBurnReviewStatus,
}

impl BatchedBudgetBurnReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedBudgetBurnReviewStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedBudgetBurnReviewStatus) -> bool {
        match self.status {
            BatchedBudgetBurnReviewStatus::Proposed => {
                match next {
                    BatchedBudgetBurnReviewStatus::Approved | BatchedBudgetBurnReviewStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedBudgetBurnReviewStatus::Approved => {
                return next == BatchedBudgetBurnReviewStatus::Rejected;
            }
            BatchedBudgetBurnReviewStatus::Rejected => {
                match next {
                    BatchedBudgetBurnReviewStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedBudgetBurnReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched budget burn review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched budget burn review", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.priority as f64
    }

    pub fn copy(&self) -> BatchedBudgetBurnReview {
        BatchedBudgetBurnReview {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            priority: self.priority,
            approved: self.approved,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a batched budget burn review in status proposed.
pub fn new_batched_budget_burn_review(code: &str, name: &str) -> BatchedBudgetBurnReview {
    BatchedBudgetBurnReview {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        priority: 0,
        approved: false,
        category: "".to_string(),
        status: BatchedBudgetBurnReviewStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_budget_burn_review("code-711", "name-603");
        assert_eq!(item.status, BatchedBudgetBurnReviewStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_budget_burn_review("code-711", "name-603");
        if item.move_to(BatchedBudgetBurnReviewStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedBudgetBurnReviewStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_budget_burn_review("code-711", "name-603");
        item.status = BatchedBudgetBurnReviewStatus::Withdrawn;
        assert!(
            item.move_to(BatchedBudgetBurnReviewStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
