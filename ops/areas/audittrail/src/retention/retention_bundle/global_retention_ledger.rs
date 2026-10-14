use super::*;

use acme_kit::text;

/// Where a global retention ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalRetentionLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalRetentionLedgerStatus {
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

impl std::fmt::Display for GlobalRetentionLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global retention ledger as the retention team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalRetentionLedger {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub audience: String,
    pub phone: String,
    pub attempts: i64,
    pub rotation_days: i64,
    pub status: GlobalRetentionLedgerStatus,
}

impl GlobalRetentionLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalRetentionLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalRetentionLedgerStatus) -> bool {
        match self.status {
            GlobalRetentionLedgerStatus::Planned => {
                match next {
                    GlobalRetentionLedgerStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRetentionLedgerStatus::Scheduled => {
                return next == GlobalRetentionLedgerStatus::Completed;
            }
            GlobalRetentionLedgerStatus::InProgress => {
                match next {
                    GlobalRetentionLedgerStatus::Completed | GlobalRetentionLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalRetentionLedgerStatus::Completed => {
                return next == GlobalRetentionLedgerStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalRetentionLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global retention ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.attempts as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalRetentionLedger {
        GlobalRetentionLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            audience: self.audience.clone(),
            phone: self.phone.clone(),
            attempts: self.attempts,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a global retention ledger in status planned.
pub fn new_global_retention_ledger(code: &str, label: &str) -> GlobalRetentionLedger {
    GlobalRetentionLedger {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        audience: "".to_string(),
        phone: "".to_string(),
        attempts: 0,
        rotation_days: 0,
        status: GlobalRetentionLedgerStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_retention_ledger("code-615", "label-587");
        assert_eq!(item.status, GlobalRetentionLedgerStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_global_retention_ledger("code-615", "label-587");
        if item.move_to(GlobalRetentionLedgerStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalRetentionLedgerStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_retention_ledger("code-615", "label-587");
        item.status = GlobalRetentionLedgerStatus::Cancelled;
        assert!(
            item.move_to(GlobalRetentionLedgerStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
