use super::*;

use acme_kit::text;

/// Where a capped page budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPageBudgetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CappedPageBudgetStatus {
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

impl std::fmt::Display for CappedPageBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped page budget as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageBudget {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub issuer: String,
    pub cost: f64,
    pub status: CappedPageBudgetStatus,
}

impl CappedPageBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPageBudgetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPageBudgetStatus) -> bool {
        match self.status {
            CappedPageBudgetStatus::Planned => {
                match next {
                    CappedPageBudgetStatus::Scheduled | CappedPageBudgetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPageBudgetStatus::Scheduled => {
                match next {
                    CappedPageBudgetStatus::InProgress | CappedPageBudgetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPageBudgetStatus::InProgress => {
                return next == CappedPageBudgetStatus::Completed || next == CappedPageBudgetStatus::Cancelled;
            }
            CappedPageBudgetStatus::Completed => {
                return next == CappedPageBudgetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedPageBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped page budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "capped page budget", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.cost - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPageBudget {
        CappedPageBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            issuer: self.issuer.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped page budget in status planned.
pub fn new_capped_page_budget(code: &str, name: &str) -> CappedPageBudget {
    CappedPageBudget {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        issuer: "".to_string(),
        cost: 0.0,
        status: CappedPageBudgetStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_capped_page_budget("code-183", "name-816");
        assert_eq!(item.status, CappedPageBudgetStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_capped_page_budget("code-183", "name-816");
        if item.move_to(CappedPageBudgetStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPageBudgetStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_capped_page_budget("code-183", "name-816");
        item.status = CappedPageBudgetStatus::Cancelled;
        assert!(
            item.move_to(CappedPageBudgetStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
