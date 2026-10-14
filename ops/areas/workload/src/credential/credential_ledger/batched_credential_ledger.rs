use super::*;

use acme_kit::text;

/// Where a batched credential ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedCredentialLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BatchedCredentialLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for BatchedCredentialLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched credential ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedCredentialLedger {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub owner_id: String,
    pub external_ref: String,
    pub tenant_id: String,
    pub host: String,
    pub active: bool,
    pub amount: f64,
    pub status: BatchedCredentialLedgerStatus,
}

impl BatchedCredentialLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedCredentialLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedCredentialLedgerStatus) -> bool {
        match self.status {
            BatchedCredentialLedgerStatus::Idle => {
                return next == BatchedCredentialLedgerStatus::Warming
                    || next == BatchedCredentialLedgerStatus::Stopped;
            }
            BatchedCredentialLedgerStatus::Warming => {
                match next {
                    BatchedCredentialLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedCredentialLedgerStatus::Ready => {
                match next {
                    BatchedCredentialLedgerStatus::Draining | BatchedCredentialLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedCredentialLedgerStatus::Draining => {
                return next == BatchedCredentialLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedCredentialLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched credential ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> BatchedCredentialLedger {
        BatchedCredentialLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            owner_id: self.owner_id.clone(),
            external_ref: self.external_ref.clone(),
            tenant_id: self.tenant_id.clone(),
            host: self.host.clone(),
            active: self.active,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched credential ledger in status idle.
pub fn new_batched_credential_ledger(code: &str, name: &str) -> BatchedCredentialLedger {
    BatchedCredentialLedger {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        owner_id: "".to_string(),
        external_ref: "".to_string(),
        tenant_id: "".to_string(),
        host: "".to_string(),
        active: false,
        amount: 0.0,
        status: BatchedCredentialLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_batched_credential_ledger("code-166", "name-222");
        assert_eq!(item.status, BatchedCredentialLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_batched_credential_ledger("code-166", "name-222");
        if item.move_to(BatchedCredentialLedgerStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedCredentialLedgerStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_batched_credential_ledger("code-166", "name-222");
        item.status = BatchedCredentialLedgerStatus::Stopped;
        assert!(
            item.move_to(BatchedCredentialLedgerStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
