use super::*;

use acme_kit::text;

/// Where a batched token ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedTokenLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedTokenLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedTokenLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a batched token ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedTokenLedger {
    pub code: String,
    pub name: String,
    pub title: String,
    pub host: String,
    pub category: String,
    pub rotation_days: i64,
    pub contact_email: String,
    pub ceiling: f64,
    pub country: String,
    pub status: BatchedTokenLedgerStatus,
}

impl BatchedTokenLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedTokenLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedTokenLedgerStatus) -> bool {
        match self.status {
            BatchedTokenLedgerStatus::Proposed => {
                return next == BatchedTokenLedgerStatus::Rejected;
            }
            BatchedTokenLedgerStatus::Approved => {
                return next == BatchedTokenLedgerStatus::Rejected || next == BatchedTokenLedgerStatus::Withdrawn;
            }
            BatchedTokenLedgerStatus::Rejected => {
                return next == BatchedTokenLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedTokenLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched token ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched token ledger", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.ceiling - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedTokenLedger {
        BatchedTokenLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            category: self.category.clone(),
            rotation_days: self.rotation_days,
            contact_email: self.contact_email.clone(),
            ceiling: self.ceiling,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a batched token ledger in status proposed.
pub fn new_batched_token_ledger(code: &str, name: &str) -> BatchedTokenLedger {
    BatchedTokenLedger {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        host: "".to_string(),
        category: "".to_string(),
        rotation_days: 0,
        contact_email: "".to_string(),
        ceiling: 0.0,
        country: "".to_string(),
        status: BatchedTokenLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_token_ledger("code-887", "name-412");
        assert_eq!(item.status, BatchedTokenLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_batched_token_ledger("code-887", "name-412");
        if item.move_to(BatchedTokenLedgerStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedTokenLedgerStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_token_ledger("code-887", "name-412");
        item.status = BatchedTokenLedgerStatus::Withdrawn;
        assert!(
            item.move_to(BatchedTokenLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
