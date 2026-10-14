use super::*;

use acme_kit::text;

/// Where a budget burn summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnSummaryStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BudgetBurnSummaryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BudgetBurnSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn summary and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnSummary {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub weight: f64,
    pub locale: String,
    pub segment: String,
    pub owner_id: String,
    pub title: String,
    pub status: BudgetBurnSummaryStatus,
}

impl BudgetBurnSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnSummaryStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnSummaryStatus) -> bool {
        match self.status {
            BudgetBurnSummaryStatus::Created => {
                match next {
                    BudgetBurnSummaryStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnSummaryStatus::Validated => {
                return next == BudgetBurnSummaryStatus::Published || next == BudgetBurnSummaryStatus::Retired;
            }
            BudgetBurnSummaryStatus::Published => {
                match next {
                    BudgetBurnSummaryStatus::Retired => {
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

    pub fn move_to(&mut self, next: BudgetBurnSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> BudgetBurnSummary {
        BudgetBurnSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            weight: self.weight,
            locale: self.locale.clone(),
            segment: self.segment.clone(),
            owner_id: self.owner_id.clone(),
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a budget burn summary in status created.
pub fn new_budget_burn_summary(code: &str, label: &str) -> BudgetBurnSummary {
    BudgetBurnSummary {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        weight: 0.0,
        locale: "".to_string(),
        segment: "".to_string(),
        owner_id: "".to_string(),
        title: "".to_string(),
        status: BudgetBurnSummaryStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_budget_burn_summary("code-816", "label-336");
        assert_eq!(item.status, BudgetBurnSummaryStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_budget_burn_summary("code-816", "label-336");
        if item.move_to(BudgetBurnSummaryStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnSummaryStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_budget_burn_summary("code-816", "label-336");
        item.status = BudgetBurnSummaryStatus::Retired;
        assert!(
            item.move_to(BudgetBurnSummaryStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
