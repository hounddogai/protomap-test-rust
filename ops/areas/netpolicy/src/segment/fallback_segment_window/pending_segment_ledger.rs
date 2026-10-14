use super::*;

use acme_kit::text;

/// Where a pending segment ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSegmentLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingSegmentLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingSegmentLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending segment ledger as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSegmentLedger {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub external_ref: String,
    pub archived: bool,
    pub minimum: f64,
    pub updated_at: i64,
    pub status: PendingSegmentLedgerStatus,
}

impl PendingSegmentLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSegmentLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSegmentLedgerStatus) -> bool {
        match self.status {
            PendingSegmentLedgerStatus::Created => {
                return next == PendingSegmentLedgerStatus::Validated || next == PendingSegmentLedgerStatus::Published;
            }
            PendingSegmentLedgerStatus::Validated => {
                match next {
                    PendingSegmentLedgerStatus::Published | PendingSegmentLedgerStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSegmentLedgerStatus::Published => {
                return next == PendingSegmentLedgerStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingSegmentLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending segment ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending segment ledger", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.minimum - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSegmentLedger {
        PendingSegmentLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            archived: self.archived,
            minimum: self.minimum,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending segment ledger in status created.
pub fn new_pending_segment_ledger(code: &str, name: &str) -> PendingSegmentLedger {
    PendingSegmentLedger {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        archived: false,
        minimum: 0.0,
        updated_at: 0,
        status: PendingSegmentLedgerStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_segment_ledger("code-957", "name-864");
        assert_eq!(item.status, PendingSegmentLedgerStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_segment_ledger("code-957", "name-864");
        if item.move_to(PendingSegmentLedgerStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSegmentLedgerStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_segment_ledger("code-957", "name-864");
        item.status = PendingSegmentLedgerStatus::Retired;
        assert!(
            item.move_to(PendingSegmentLedgerStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
