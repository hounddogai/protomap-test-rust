use super::*;

use acme_kit::text;

/// Where a identity ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityLedgerStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IdentityLedgerStatus {
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

impl std::fmt::Display for IdentityLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a identity ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityLedger {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub ends_at: i64,
    pub fingerprint: String,
    pub status: IdentityLedgerStatus,
}

impl IdentityLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityLedgerStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityLedgerStatus) -> bool {
        match self.status {
            IdentityLedgerStatus::Idle => {
                return next == IdentityLedgerStatus::Warming || next == IdentityLedgerStatus::Draining;
            }
            IdentityLedgerStatus::Warming => {
                return next == IdentityLedgerStatus::Draining || next == IdentityLedgerStatus::Stopped;
            }
            IdentityLedgerStatus::Ready => {
                match next {
                    IdentityLedgerStatus::Draining | IdentityLedgerStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityLedgerStatus::Draining => {
                return next == IdentityLedgerStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> IdentityLedger {
        IdentityLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            ends_at: self.ends_at,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a identity ledger in status idle.
pub fn new_identity_ledger(code: &str, name: &str) -> IdentityLedger {
    IdentityLedger {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        ends_at: 0,
        fingerprint: "".to_string(),
        status: IdentityLedgerStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_identity_ledger("code-340", "name-258");
        assert_eq!(item.status, IdentityLedgerStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_identity_ledger("code-340", "name-258");
        if item.move_to(IdentityLedgerStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityLedgerStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_identity_ledger("code-340", "name-258");
        item.status = IdentityLedgerStatus::Stopped;
        assert!(
            item.move_to(IdentityLedgerStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
