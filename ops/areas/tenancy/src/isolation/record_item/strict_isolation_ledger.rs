use super::*;

use acme_kit::text;

/// Where a strict isolation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictIsolationLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictIsolationLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictIsolationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict isolation ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictIsolationLedger {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub segment: String,
    pub window_days: i64,
    pub status: StrictIsolationLedgerStatus,
}

impl StrictIsolationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictIsolationLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictIsolationLedgerStatus) -> bool {
        match self.status {
            StrictIsolationLedgerStatus::Proposed => {
                match next {
                    StrictIsolationLedgerStatus::Approved | StrictIsolationLedgerStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictIsolationLedgerStatus::Approved => {
                match next {
                    StrictIsolationLedgerStatus::Rejected | StrictIsolationLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictIsolationLedgerStatus::Rejected => {
                match next {
                    StrictIsolationLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: StrictIsolationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict isolation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.ceiling * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictIsolationLedger {
        StrictIsolationLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            segment: self.segment.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a strict isolation ledger in status proposed.
pub fn new_strict_isolation_ledger(code: &str, name: &str) -> StrictIsolationLedger {
    StrictIsolationLedger {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        segment: "".to_string(),
        window_days: 0,
        status: StrictIsolationLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_strict_isolation_ledger("code-989", "name-669");
        assert_eq!(item.status, StrictIsolationLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_strict_isolation_ledger("code-989", "name-669");
        if item.move_to(StrictIsolationLedgerStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictIsolationLedgerStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_strict_isolation_ledger("code-989", "name-669");
        item.status = StrictIsolationLedgerStatus::Withdrawn;
        assert!(
            item.move_to(StrictIsolationLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
