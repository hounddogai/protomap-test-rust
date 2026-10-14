use super::*;

use acme_kit::text;

/// Where a delegation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DelegationLedgerStatus {
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

impl std::fmt::Display for DelegationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationLedger {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub burst: i64,
    pub batch_size: i64,
    pub priority: i64,
    pub status: DelegationLedgerStatus,
}

impl DelegationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationLedgerStatus) -> bool {
        match self.status {
            DelegationLedgerStatus::New => {
                match next {
                    DelegationLedgerStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationLedgerStatus::Queued => {
                match next {
                    DelegationLedgerStatus::Done | DelegationLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationLedgerStatus::Running => {
                return next == DelegationLedgerStatus::Failed;
            }
            DelegationLedgerStatus::Done => {
                return next == DelegationLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.priority as f64 - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationLedger {
        DelegationLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            burst: self.burst,
            batch_size: self.batch_size,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation ledger in status new.
pub fn new_delegation_ledger(code: &str, label: &str) -> DelegationLedger {
    DelegationLedger {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        burst: 0,
        batch_size: 0,
        priority: 0,
        status: DelegationLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_delegation_ledger("code-247", "label-559");
        assert_eq!(item.status, DelegationLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_delegation_ledger("code-247", "label-559");
        if item.move_to(DelegationLedgerStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DelegationLedgerStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_delegation_ledger("code-247", "label-559");
        item.status = DelegationLedgerStatus::Failed;
        assert!(
            item.move_to(DelegationLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
