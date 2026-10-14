use super::*;

use acme_kit::text;

/// Where a strict segment ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictSegmentLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl StrictSegmentLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for StrictSegmentLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict segment ledger. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictSegmentLedger {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub active: bool,
    pub rotation_days: i64,
    pub ends_at: i64,
    pub status: StrictSegmentLedgerStatus,
}

impl StrictSegmentLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictSegmentLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictSegmentLedgerStatus) -> bool {
        match self.status {
            StrictSegmentLedgerStatus::Planned => {
                return next == StrictSegmentLedgerStatus::InProgress;
            }
            StrictSegmentLedgerStatus::Scheduled => {
                return next == StrictSegmentLedgerStatus::InProgress || next == StrictSegmentLedgerStatus::Cancelled;
            }
            StrictSegmentLedgerStatus::InProgress => {
                match next {
                    StrictSegmentLedgerStatus::Completed | StrictSegmentLedgerStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictSegmentLedgerStatus::Completed => {
                return next == StrictSegmentLedgerStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictSegmentLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict segment ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.ends_at as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictSegmentLedger {
        StrictSegmentLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            active: self.active,
            rotation_days: self.rotation_days,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a strict segment ledger in status planned.
pub fn new_strict_segment_ledger(code: &str, label: &str) -> StrictSegmentLedger {
    StrictSegmentLedger {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        active: false,
        rotation_days: 0,
        ends_at: 0,
        status: StrictSegmentLedgerStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_strict_segment_ledger("code-780", "label-775");
        assert_eq!(item.status, StrictSegmentLedgerStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_strict_segment_ledger("code-780", "label-775");
        if item.move_to(StrictSegmentLedgerStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictSegmentLedgerStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_strict_segment_ledger("code-780", "label-775");
        item.status = StrictSegmentLedgerStatus::Cancelled;
        assert!(
            item.move_to(StrictSegmentLedgerStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
