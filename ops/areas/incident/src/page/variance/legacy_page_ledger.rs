use super::*;

use acme_kit::text;

/// Where a legacy page ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyPageLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyPageLedgerStatus {
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

impl std::fmt::Display for LegacyPageLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy page ledger as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyPageLedger {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub starts_at: i64,
    pub checksum: String,
    pub ratio: f64,
    pub ceiling: f64,
    pub batch_size: i64,
    pub created_at: i64,
    pub weight: f64,
    pub quota_units: i64,
    pub status: LegacyPageLedgerStatus,
}

impl LegacyPageLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyPageLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyPageLedgerStatus) -> bool {
        match self.status {
            LegacyPageLedgerStatus::Open => {
                return next == LegacyPageLedgerStatus::InReview || next == LegacyPageLedgerStatus::Archived;
            }
            LegacyPageLedgerStatus::InReview => {
                return next == LegacyPageLedgerStatus::Reopened;
            }
            LegacyPageLedgerStatus::Resolved => {
                match next {
                    LegacyPageLedgerStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyPageLedgerStatus::Reopened => {
                return next == LegacyPageLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyPageLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy page ledger {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.created_at as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyPageLedger {
        LegacyPageLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            starts_at: self.starts_at,
            checksum: self.checksum.clone(),
            ratio: self.ratio,
            ceiling: self.ceiling,
            batch_size: self.batch_size,
            created_at: self.created_at,
            weight: self.weight,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy page ledger in status open.
pub fn new_legacy_page_ledger(code: &str, label: &str) -> LegacyPageLedger {
    LegacyPageLedger {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        starts_at: 0,
        checksum: "".to_string(),
        ratio: 0.0,
        ceiling: 0.0,
        batch_size: 0,
        created_at: 0,
        weight: 0.0,
        quota_units: 0,
        status: LegacyPageLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_page_ledger("code-108", "label-772");
        assert_eq!(item.status, LegacyPageLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_legacy_page_ledger("code-108", "label-772");
        if item.move_to(LegacyPageLedgerStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyPageLedgerStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_page_ledger("code-108", "label-772");
        item.status = LegacyPageLedgerStatus::Archived;
        assert!(
            item.move_to(LegacyPageLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
