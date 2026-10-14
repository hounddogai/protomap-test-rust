use super::*;

use acme_kit::text;

/// Where a manual ttl policy ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualTtlPolicyLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ManualTtlPolicyLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for ManualTtlPolicyLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual ttl policy ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualTtlPolicyLedger {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub unit: String,
    pub key_id: String,
    pub archived: bool,
    pub score: f64,
    pub status: ManualTtlPolicyLedgerStatus,
}

impl ManualTtlPolicyLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualTtlPolicyLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualTtlPolicyLedgerStatus) -> bool {
        match self.status {
            ManualTtlPolicyLedgerStatus::Open => {
                return next == ManualTtlPolicyLedgerStatus::Reopened || next == ManualTtlPolicyLedgerStatus::Archived;
            }
            ManualTtlPolicyLedgerStatus::InReview => {
                return next == ManualTtlPolicyLedgerStatus::Resolved || next == ManualTtlPolicyLedgerStatus::Reopened;
            }
            ManualTtlPolicyLedgerStatus::Resolved => {
                match next {
                    ManualTtlPolicyLedgerStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualTtlPolicyLedgerStatus::Reopened => {
                return next == ManualTtlPolicyLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualTtlPolicyLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual ttl policy ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> ManualTtlPolicyLedger {
        ManualTtlPolicyLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            unit: self.unit.clone(),
            key_id: self.key_id.clone(),
            archived: self.archived,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a manual ttl policy ledger in status open.
pub fn new_manual_ttl_policy_ledger(code: &str, name: &str) -> ManualTtlPolicyLedger {
    ManualTtlPolicyLedger {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        unit: "".to_string(),
        key_id: "".to_string(),
        archived: false,
        score: 0.0,
        status: ManualTtlPolicyLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_manual_ttl_policy_ledger("code-106", "name-282");
        assert_eq!(item.status, ManualTtlPolicyLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_manual_ttl_policy_ledger("code-106", "name-282");
        if item.move_to(ManualTtlPolicyLedgerStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualTtlPolicyLedgerStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_manual_ttl_policy_ledger("code-106", "name-282");
        item.status = ManualTtlPolicyLedgerStatus::Archived;
        assert!(
            item.move_to(ManualTtlPolicyLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
