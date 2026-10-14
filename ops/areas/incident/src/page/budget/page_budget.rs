use super::*;

use acme_kit::text;

/// Where a page budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PageBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PageBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a page budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PageBudget {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub cost: f64,
    pub category: String,
    pub status: PageBudgetStatus,
}

impl PageBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageBudgetStatus) -> bool {
        match self.status {
            PageBudgetStatus::Planned => {
                match next {
                    PageBudgetStatus::Scheduled | PageBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageBudgetStatus::Scheduled => {
                return next == PageBudgetStatus::Cancelled;
            }
            PageBudgetStatus::InProgress => {
                match next {
                    PageBudgetStatus::Completed | PageBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageBudgetStatus::Completed => {
                return next == PageBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "page budget", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.cost + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageBudget {
        PageBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            cost: self.cost,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a page budget in status planned.
pub fn new_page_budget(code: &str, label: &str) -> PageBudget {
    PageBudget {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        cost: 0.0,
        category: "".to_string(),
        status: PageBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_page_budget("code-896", "label-807");
        assert_eq!(item.status, PageBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_page_budget("code-896", "label-807");
        if item.move_to(PageBudgetStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageBudgetStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_page_budget("code-896", "label-807");
        item.status = PageBudgetStatus::Cancelled;
        assert!(
            item.move_to(PageBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
