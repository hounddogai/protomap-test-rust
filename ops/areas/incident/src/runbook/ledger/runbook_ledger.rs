use super::*;

use acme_kit::text;

/// Where a runbook ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RunbookLedgerStatus {
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

impl std::fmt::Display for RunbookLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookLedger {
    pub code: String,
    pub name: String,
    pub expires_at: i64,
    pub country: String,
    pub active: bool,
    pub key_id: String,
    pub status: RunbookLedgerStatus,
}

impl RunbookLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookLedgerStatus) -> bool {
        match self.status {
            RunbookLedgerStatus::Planned => {
                match next {
                    RunbookLedgerStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookLedgerStatus::Scheduled => {
                match next {
                    RunbookLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookLedgerStatus::InProgress => {
                return next == RunbookLedgerStatus::Completed;
            }
            RunbookLedgerStatus::Completed => {
                return next == RunbookLedgerStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> RunbookLedger {
        RunbookLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            country: self.country.clone(),
            active: self.active,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a runbook ledger in status planned.
pub fn new_runbook_ledger(code: &str, name: &str) -> RunbookLedger {
    RunbookLedger {
        code: code.to_string(),
        name: name.to_string(),
        expires_at: 0,
        country: "".to_string(),
        active: false,
        key_id: "".to_string(),
        status: RunbookLedgerStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_runbook_ledger("code-734", "name-671");
        assert_eq!(item.status, RunbookLedgerStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_runbook_ledger("code-734", "name-671");
        if item.move_to(RunbookLedgerStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RunbookLedgerStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_runbook_ledger("code-734", "name-671");
        item.status = RunbookLedgerStatus::Cancelled;
        assert!(
            item.move_to(RunbookLedgerStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
