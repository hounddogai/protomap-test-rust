use super::*;

use acme_kit::text;

/// Where a pending budget burn record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingBudgetBurnRecordStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingBudgetBurnRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingBudgetBurnRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending budget burn record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingBudgetBurnRecord {
    pub code: String,
    pub name: String,
    pub label: String,
    pub source: String,
    pub sequence: i64,
    pub window_days: i64,
    pub contact_email: String,
    pub burst: i64,
    pub scope: String,
    pub grace_days: i64,
    pub attempts: i64,
    pub status: PendingBudgetBurnRecordStatus,
}

impl PendingBudgetBurnRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingBudgetBurnRecordStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingBudgetBurnRecordStatus) -> bool {
        match self.status {
            PendingBudgetBurnRecordStatus::Created => {
                match next {
                    PendingBudgetBurnRecordStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingBudgetBurnRecordStatus::Validated => {
                match next {
                    PendingBudgetBurnRecordStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingBudgetBurnRecordStatus::Published => {
                return next == PendingBudgetBurnRecordStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingBudgetBurnRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending budget burn record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "pending budget burn record", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.attempts as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingBudgetBurnRecord {
        PendingBudgetBurnRecord {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            sequence: self.sequence,
            window_days: self.window_days,
            contact_email: self.contact_email.clone(),
            burst: self.burst,
            scope: self.scope.clone(),
            grace_days: self.grace_days,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending budget burn record in status created.
pub fn new_pending_budget_burn_record(code: &str, name: &str) -> PendingBudgetBurnRecord {
    PendingBudgetBurnRecord {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        source: "".to_string(),
        sequence: 0,
        window_days: 0,
        contact_email: "".to_string(),
        burst: 0,
        scope: "".to_string(),
        grace_days: 0,
        attempts: 0,
        status: PendingBudgetBurnRecordStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_budget_burn_record("code-949", "name-84");
        assert_eq!(item.status, PendingBudgetBurnRecordStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_pending_budget_burn_record("code-949", "name-84");
        if item.move_to(PendingBudgetBurnRecordStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingBudgetBurnRecordStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_budget_burn_record("code-949", "name-84");
        item.status = PendingBudgetBurnRecordStatus::Retired;
        assert!(
            item.move_to(PendingBudgetBurnRecordStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
