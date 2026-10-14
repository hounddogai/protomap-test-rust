use super::*;

use acme_kit::text;

/// Where a budget burn item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnItemStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BudgetBurnItemStatus {
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

impl std::fmt::Display for BudgetBurnItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn item and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnItem {
    pub code: String,
    pub label: String,
    pub source: String,
    pub ttl_seconds: i64,
    pub maximum: f64,
    pub status: BudgetBurnItemStatus,
}

impl BudgetBurnItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnItemStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnItemStatus) -> bool {
        match self.status {
            BudgetBurnItemStatus::Planned => {
                match next {
                    BudgetBurnItemStatus::Scheduled | BudgetBurnItemStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnItemStatus::Scheduled => {
                return next == BudgetBurnItemStatus::InProgress || next == BudgetBurnItemStatus::Cancelled;
            }
            BudgetBurnItemStatus::InProgress => {
                match next {
                    BudgetBurnItemStatus::Completed | BudgetBurnItemStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnItemStatus::Completed => {
                match next {
                    BudgetBurnItemStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: BudgetBurnItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "budget burn item", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.maximum + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnItem {
        BudgetBurnItem {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            ttl_seconds: self.ttl_seconds,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a budget burn item in status planned.
pub fn new_budget_burn_item(code: &str, label: &str) -> BudgetBurnItem {
    BudgetBurnItem {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        ttl_seconds: 0,
        maximum: 0.0,
        status: BudgetBurnItemStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_budget_burn_item("code-633", "label-216");
        assert_eq!(item.status, BudgetBurnItemStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_budget_burn_item("code-633", "label-216");
        if item.move_to(BudgetBurnItemStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnItemStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_budget_burn_item("code-633", "label-216");
        item.status = BudgetBurnItemStatus::Cancelled;
        assert!(
            item.move_to(BudgetBurnItemStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
