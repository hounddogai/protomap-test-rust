use super::*;

use acme_kit::text;

/// Where a budget burn entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnEntryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BudgetBurnEntryStatus {
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

impl std::fmt::Display for BudgetBurnEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn entry. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnEntry {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub window_days: i64,
    pub tenant_id: String,
    pub label: String,
    pub source: String,
    pub priority: i64,
    pub key_id: String,
    pub cost: f64,
    pub audience: String,
    pub status: BudgetBurnEntryStatus,
}

impl BudgetBurnEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnEntryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnEntryStatus) -> bool {
        match self.status {
            BudgetBurnEntryStatus::New => {
                match next {
                    BudgetBurnEntryStatus::Queued | BudgetBurnEntryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnEntryStatus::Queued => {
                return next == BudgetBurnEntryStatus::Done;
            }
            BudgetBurnEntryStatus::Running => {
                return next == BudgetBurnEntryStatus::Done || next == BudgetBurnEntryStatus::Failed;
            }
            BudgetBurnEntryStatus::Done => {
                return next == BudgetBurnEntryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.cost * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnEntry {
        BudgetBurnEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            window_days: self.window_days,
            tenant_id: self.tenant_id.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            priority: self.priority,
            key_id: self.key_id.clone(),
            cost: self.cost,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a budget burn entry in status new.
pub fn new_budget_burn_entry(code: &str, name: &str) -> BudgetBurnEntry {
    BudgetBurnEntry {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        window_days: 0,
        tenant_id: "".to_string(),
        label: "".to_string(),
        source: "".to_string(),
        priority: 0,
        key_id: "".to_string(),
        cost: 0.0,
        audience: "".to_string(),
        status: BudgetBurnEntryStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_budget_burn_entry("code-16", "name-423");
        assert_eq!(item.status, BudgetBurnEntryStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_budget_burn_entry("code-16", "name-423");
        if item.move_to(BudgetBurnEntryStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnEntryStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_budget_burn_entry("code-16", "name-423");
        item.status = BudgetBurnEntryStatus::Failed;
        assert!(
            item.move_to(BudgetBurnEntryStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
