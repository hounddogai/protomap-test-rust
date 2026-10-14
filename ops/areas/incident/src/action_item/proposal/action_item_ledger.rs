use super::*;

use acme_kit::text;

/// Where a action item ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ActionItemLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ActionItemLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A action item ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemLedger {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub source: String,
    pub issuer: String,
    pub quota_units: i64,
    pub country: String,
    pub notes: String,
    pub port: i64,
    pub margin: f64,
    pub status: ActionItemLedgerStatus,
}

impl ActionItemLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemLedgerStatus) -> bool {
        match self.status {
            ActionItemLedgerStatus::Proposed => {
                return next == ActionItemLedgerStatus::Rejected;
            }
            ActionItemLedgerStatus::Approved => {
                match next {
                    ActionItemLedgerStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemLedgerStatus::Rejected => {
                return next == ActionItemLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "action item ledger", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.margin * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemLedger {
        ActionItemLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            country: self.country.clone(),
            notes: self.notes.clone(),
            port: self.port,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a action item ledger in status proposed.
pub fn new_action_item_ledger(code: &str, title: &str) -> ActionItemLedger {
    ActionItemLedger {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        issuer: "".to_string(),
        quota_units: 0,
        country: "".to_string(),
        notes: "".to_string(),
        port: 0,
        margin: 0.0,
        status: ActionItemLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_action_item_ledger("code-233", "title-677");
        assert_eq!(item.status, ActionItemLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_action_item_ledger("code-233", "title-677");
        if item.move_to(ActionItemLedgerStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ActionItemLedgerStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_action_item_ledger("code-233", "title-677");
        item.status = ActionItemLedgerStatus::Withdrawn;
        assert!(
            item.move_to(ActionItemLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
