use super::*;

use acme_kit::text;

/// Where a capped grant ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedGrantLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CappedGrantLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CappedGrantLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One capped grant ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedGrantLedger {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub pinned: bool,
    pub floor_value: f64,
    pub fingerprint: String,
    pub audience: String,
    pub channel: String,
    pub starts_at: i64,
    pub status: CappedGrantLedgerStatus,
}

impl CappedGrantLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedGrantLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedGrantLedgerStatus) -> bool {
        match self.status {
            CappedGrantLedgerStatus::Proposed => {
                match next {
                    CappedGrantLedgerStatus::Rejected | CappedGrantLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedGrantLedgerStatus::Approved => {
                return next == CappedGrantLedgerStatus::Withdrawn;
            }
            CappedGrantLedgerStatus::Rejected => {
                return next == CappedGrantLedgerStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedGrantLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped grant ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.priority as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedGrantLedger {
        CappedGrantLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            pinned: self.pinned,
            floor_value: self.floor_value,
            fingerprint: self.fingerprint.clone(),
            audience: self.audience.clone(),
            channel: self.channel.clone(),
            starts_at: self.starts_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a capped grant ledger in status proposed.
pub fn new_capped_grant_ledger(code: &str, label: &str) -> CappedGrantLedger {
    CappedGrantLedger {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        pinned: false,
        floor_value: 0.0,
        fingerprint: "".to_string(),
        audience: "".to_string(),
        channel: "".to_string(),
        starts_at: 0,
        status: CappedGrantLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_capped_grant_ledger("code-368", "label-740");
        assert_eq!(item.status, CappedGrantLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_capped_grant_ledger("code-368", "label-740");
        if item.move_to(CappedGrantLedgerStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedGrantLedgerStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_capped_grant_ledger("code-368", "label-740");
        item.status = CappedGrantLedgerStatus::Withdrawn;
        assert!(
            item.move_to(CappedGrantLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
