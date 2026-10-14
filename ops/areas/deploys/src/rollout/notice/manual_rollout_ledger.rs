use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual rollout ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRolloutLedgerStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualRolloutLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualRolloutLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a manual rollout ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRolloutLedger {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub approved: bool,
    pub window_days: i64,
    pub sequence: i64,
    pub duration_minutes: i64,
    pub threshold: f64,
    pub title: String,
    pub revision: i64,
    pub status: ManualRolloutLedgerStatus,
}

impl ManualRolloutLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRolloutLedgerStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRolloutLedgerStatus) -> bool {
        match self.status {
            ManualRolloutLedgerStatus::Requested => {
                return next == ManualRolloutLedgerStatus::Granted || next == ManualRolloutLedgerStatus::Expired;
            }
            ManualRolloutLedgerStatus::Granted => {
                return next == ManualRolloutLedgerStatus::Revoked || next == ManualRolloutLedgerStatus::Expired;
            }
            ManualRolloutLedgerStatus::Revoked => {
                return next == ManualRolloutLedgerStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualRolloutLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual rollout ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.revision as f64 - (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRolloutLedger {
        ManualRolloutLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            approved: self.approved,
            window_days: self.window_days,
            sequence: self.sequence,
            duration_minutes: self.duration_minutes,
            threshold: self.threshold,
            title: self.title.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual rollout ledger in status requested.
pub fn new_manual_rollout_ledger(code: &str, name: &str) -> ManualRolloutLedger {
    ManualRolloutLedger {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        approved: false,
        window_days: 0,
        sequence: 0,
        duration_minutes: 0,
        threshold: 0.0,
        title: "".to_string(),
        revision: 0,
        status: ManualRolloutLedgerStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_rollout_ledger("code-451", "name-685");
        assert_eq!(item.status, ManualRolloutLedgerStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_rollout_ledger("code-451", "name-685");
        if item.move_to(ManualRolloutLedgerStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRolloutLedgerStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_rollout_ledger("code-451", "name-685");
        item.status = ManualRolloutLedgerStatus::Expired;
        assert!(
            item.move_to(ManualRolloutLedgerStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
