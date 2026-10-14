use super::*;

use acme_kit::text;

/// Where a batched health check ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedHealthCheckLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl BatchedHealthCheckLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for BatchedHealthCheckLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched health check ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckLedger {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub margin: f64,
    pub scope: String,
    pub duration_minutes: i64,
    pub status: BatchedHealthCheckLedgerStatus,
}

impl BatchedHealthCheckLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedHealthCheckLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedHealthCheckLedgerStatus) -> bool {
        match self.status {
            BatchedHealthCheckLedgerStatus::Requested => {
                return next == BatchedHealthCheckLedgerStatus::Revoked;
            }
            BatchedHealthCheckLedgerStatus::Granted => {
                return next == BatchedHealthCheckLedgerStatus::Revoked;
            }
            BatchedHealthCheckLedgerStatus::Revoked => {
                return next == BatchedHealthCheckLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedHealthCheckLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched health check ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        (self.margin * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedHealthCheckLedger {
        BatchedHealthCheckLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            margin: self.margin,
            scope: self.scope.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a batched health check ledger in status requested.
pub fn new_batched_health_check_ledger(code: &str, name: &str) -> BatchedHealthCheckLedger {
    BatchedHealthCheckLedger {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        margin: 0.0,
        scope: "".to_string(),
        duration_minutes: 0,
        status: BatchedHealthCheckLedgerStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_batched_health_check_ledger("code-291", "name-469");
        assert_eq!(item.status, BatchedHealthCheckLedgerStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_batched_health_check_ledger("code-291", "name-469");
        if item.move_to(BatchedHealthCheckLedgerStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedHealthCheckLedgerStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_batched_health_check_ledger("code-291", "name-469");
        item.status = BatchedHealthCheckLedgerStatus::Expired;
        assert!(
            item.move_to(BatchedHealthCheckLedgerStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
