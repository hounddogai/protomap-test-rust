use super::*;

use acme_kit::text;

/// Where a pending region pin ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRegionPinLedgerStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PendingRegionPinLedgerStatus {
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

impl std::fmt::Display for PendingRegionPinLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending region pin ledger as the region pin team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRegionPinLedger {
    pub code: String,
    pub name: String,
    pub host: String,
    pub maximum: f64,
    pub minimum: f64,
    pub margin: f64,
    pub status: PendingRegionPinLedgerStatus,
}

impl PendingRegionPinLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRegionPinLedgerStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRegionPinLedgerStatus) -> bool {
        match self.status {
            PendingRegionPinLedgerStatus::New => {
                return next == PendingRegionPinLedgerStatus::Queued;
            }
            PendingRegionPinLedgerStatus::Queued => {
                return next == PendingRegionPinLedgerStatus::Running || next == PendingRegionPinLedgerStatus::Failed;
            }
            PendingRegionPinLedgerStatus::Running => {
                match next {
                    PendingRegionPinLedgerStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingRegionPinLedgerStatus::Done => {
                return next == PendingRegionPinLedgerStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRegionPinLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending region pin ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending region pin ledger", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.minimum + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRegionPinLedger {
        PendingRegionPinLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            maximum: self.maximum,
            minimum: self.minimum,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending region pin ledger in status new.
pub fn new_pending_region_pin_ledger(code: &str, name: &str) -> PendingRegionPinLedger {
    PendingRegionPinLedger {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        maximum: 0.0,
        minimum: 0.0,
        margin: 0.0,
        status: PendingRegionPinLedgerStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_pending_region_pin_ledger("code-905", "name-83");
        assert_eq!(item.status, PendingRegionPinLedgerStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_pending_region_pin_ledger("code-905", "name-83");
        if item.move_to(PendingRegionPinLedgerStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRegionPinLedgerStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_pending_region_pin_ledger("code-905", "name-83");
        item.status = PendingRegionPinLedgerStatus::Failed;
        assert!(
            item.move_to(PendingRegionPinLedgerStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
