use super::*;

use acme_kit::text;

/// Where a alert ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AlertLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AlertLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AlertLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A alert ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AlertLedger {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub rotation_days: i64,
    pub checksum: String,
    pub status: AlertLedgerStatus,
}

impl AlertLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AlertLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AlertLedgerStatus) -> bool {
        match self.status {
            AlertLedgerStatus::Proposed => {
                match next {
                    AlertLedgerStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertLedgerStatus::Approved => {
                match next {
                    AlertLedgerStatus::Rejected | AlertLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AlertLedgerStatus::Rejected => {
                match next {
                    AlertLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AlertLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "alert ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.rotation_days as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AlertLedger {
        AlertLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            rotation_days: self.rotation_days,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a alert ledger in status proposed.
pub fn new_alert_ledger(code: &str, label: &str) -> AlertLedger {
    AlertLedger {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        rotation_days: 0,
        checksum: "".to_string(),
        status: AlertLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_alert_ledger("code-997", "label-261");
        assert_eq!(item.status, AlertLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_alert_ledger("code-997", "label-261");
        if item.move_to(AlertLedgerStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AlertLedgerStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_alert_ledger("code-997", "label-261");
        item.status = AlertLedgerStatus::Withdrawn;
        assert!(
            item.move_to(AlertLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
