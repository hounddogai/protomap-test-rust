use super::*;

use acme_kit::text;

/// Where a pending action item entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingActionItemEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingActionItemEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingActionItemEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending action item entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingActionItemEntry {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub unit: String,
    pub rate: f64,
    pub retries: i64,
    pub status: PendingActionItemEntryStatus,
}

impl PendingActionItemEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingActionItemEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingActionItemEntryStatus) -> bool {
        match self.status {
            PendingActionItemEntryStatus::Requested => {
                match next {
                    PendingActionItemEntryStatus::Revoked | PendingActionItemEntryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingActionItemEntryStatus::Granted => {
                return next == PendingActionItemEntryStatus::Revoked || next == PendingActionItemEntryStatus::Expired;
            }
            PendingActionItemEntryStatus::Revoked => {
                match next {
                    PendingActionItemEntryStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingActionItemEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending action item entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.retries as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingActionItemEntry {
        PendingActionItemEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            unit: self.unit.clone(),
            rate: self.rate,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a pending action item entry in status requested.
pub fn new_pending_action_item_entry(code: &str, label: &str) -> PendingActionItemEntry {
    PendingActionItemEntry {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        unit: "".to_string(),
        rate: 0.0,
        retries: 0,
        status: PendingActionItemEntryStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_action_item_entry("code-595", "label-376");
        assert_eq!(item.status, PendingActionItemEntryStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_action_item_entry("code-595", "label-376");
        if item.move_to(PendingActionItemEntryStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingActionItemEntryStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_action_item_entry("code-595", "label-376");
        item.status = PendingActionItemEntryStatus::Expired;
        assert!(
            item.move_to(PendingActionItemEntryStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
