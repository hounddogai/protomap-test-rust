use super::*;

use acme_kit::text;

/// Where a budget burn review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BudgetBurnReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BudgetBurnReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnReview {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub budget: f64,
    pub principal: String,
    pub maximum: f64,
    pub fingerprint: String,
    pub locale: String,
    pub status: BudgetBurnReviewStatus,
}

impl BudgetBurnReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnReviewStatus) -> bool {
        match self.status {
            BudgetBurnReviewStatus::Created => {
                return next == BudgetBurnReviewStatus::Published;
            }
            BudgetBurnReviewStatus::Validated => {
                return next == BudgetBurnReviewStatus::Published || next == BudgetBurnReviewStatus::Retired;
            }
            BudgetBurnReviewStatus::Published => {
                match next {
                    BudgetBurnReviewStatus::Retired => {
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

    pub fn move_to(&mut self, next: BudgetBurnReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "budget burn review", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.maximum + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnReview {
        BudgetBurnReview {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            budget: self.budget,
            principal: self.principal.clone(),
            maximum: self.maximum,
            fingerprint: self.fingerprint.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a budget burn review in status created.
pub fn new_budget_burn_review(code: &str, title: &str) -> BudgetBurnReview {
    BudgetBurnReview {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        budget: 0.0,
        principal: "".to_string(),
        maximum: 0.0,
        fingerprint: "".to_string(),
        locale: "".to_string(),
        status: BudgetBurnReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_budget_burn_review("code-454", "title-984");
        assert_eq!(item.status, BudgetBurnReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_budget_burn_review("code-454", "title-984");
        if item.move_to(BudgetBurnReviewStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnReviewStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_budget_burn_review("code-454", "title-984");
        item.status = BudgetBurnReviewStatus::Retired;
        assert!(
            item.move_to(BudgetBurnReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
