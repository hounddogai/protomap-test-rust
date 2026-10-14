use super::*;

use acme_kit::text;

/// Where a manual release ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualReleaseLedgerStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualReleaseLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualReleaseLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual release ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualReleaseLedger {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub window_days: i64,
    pub cipher: String,
    pub starts_at: i64,
    pub category: String,
    pub unit: String,
    pub visible: bool,
    pub status: ManualReleaseLedgerStatus,
}

impl ManualReleaseLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualReleaseLedgerStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualReleaseLedgerStatus) -> bool {
        match self.status {
            ManualReleaseLedgerStatus::Proposed => {
                match next {
                    ManualReleaseLedgerStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualReleaseLedgerStatus::Approved => {
                return next == ManualReleaseLedgerStatus::Rejected;
            }
            ManualReleaseLedgerStatus::Rejected => {
                match next {
                    ManualReleaseLedgerStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualReleaseLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual release ledger {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualReleaseLedger {
        ManualReleaseLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            window_days: self.window_days,
            cipher: self.cipher.clone(),
            starts_at: self.starts_at,
            category: self.category.clone(),
            unit: self.unit.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a manual release ledger in status proposed.
pub fn new_manual_release_ledger(code: &str, title: &str) -> ManualReleaseLedger {
    ManualReleaseLedger {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        window_days: 0,
        cipher: "".to_string(),
        starts_at: 0,
        category: "".to_string(),
        unit: "".to_string(),
        visible: false,
        status: ManualReleaseLedgerStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_release_ledger("code-57", "title-627");
        assert_eq!(item.status, ManualReleaseLedgerStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_manual_release_ledger("code-57", "title-627");
        if item.move_to(ManualReleaseLedgerStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualReleaseLedgerStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_release_ledger("code-57", "title-627");
        item.status = ManualReleaseLedgerStatus::Withdrawn;
        assert!(
            item.move_to(ManualReleaseLedgerStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
