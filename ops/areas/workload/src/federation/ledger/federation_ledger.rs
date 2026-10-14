use super::*;

use acme_kit::text;

/// Where a federation ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationLedgerStatus {
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

impl std::fmt::Display for FederationLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationLedger {
    pub code: String,
    pub name: String,
    pub description: String,
    pub floor_value: f64,
    pub key_id: String,
    pub maximum: f64,
    pub status: FederationLedgerStatus,
}

impl FederationLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationLedgerStatus) -> bool {
        match self.status {
            FederationLedgerStatus::New => {
                return next == FederationLedgerStatus::Running || next == FederationLedgerStatus::Failed;
            }
            FederationLedgerStatus::Queued => {
                match next {
                    FederationLedgerStatus::Running | FederationLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationLedgerStatus::Running => {
                return next == FederationLedgerStatus::Failed;
            }
            FederationLedgerStatus::Done => {
                return next == FederationLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.maximum - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationLedger {
        FederationLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            floor_value: self.floor_value,
            key_id: self.key_id.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a federation ledger in status new.
pub fn new_federation_ledger(code: &str, name: &str) -> FederationLedger {
    FederationLedger {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        floor_value: 0.0,
        key_id: "".to_string(),
        maximum: 0.0,
        status: FederationLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_federation_ledger("code-655", "name-31");
        assert_eq!(item.status, FederationLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_federation_ledger("code-655", "name-31");
        if item.move_to(FederationLedgerStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationLedgerStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_federation_ledger("code-655", "name-31");
        item.status = FederationLedgerStatus::Failed;
        assert!(
            item.move_to(FederationLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
