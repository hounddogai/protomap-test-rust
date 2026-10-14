use super::*;

use acme_kit::text;

/// Where a timeline ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineLedgerStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TimelineLedgerStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TimelineLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineLedger {
    pub code: String,
    pub title: String,
    pub description: String,
    pub port: i64,
    pub capacity: i64,
    pub weight: f64,
    pub status: TimelineLedgerStatus,
}

impl TimelineLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineLedgerStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineLedgerStatus) -> bool {
        match self.status {
            TimelineLedgerStatus::Created => {
                return next == TimelineLedgerStatus::Retired;
            }
            TimelineLedgerStatus::Validated => {
                return next == TimelineLedgerStatus::Published;
            }
            TimelineLedgerStatus::Published => {
                match next {
                    TimelineLedgerStatus::Retired => {
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

    pub fn move_to(&mut self, next: TimelineLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.capacity as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TimelineLedger {
        TimelineLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            port: self.port,
            capacity: self.capacity,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a timeline ledger in status created.
pub fn new_timeline_ledger(code: &str, title: &str) -> TimelineLedger {
    TimelineLedger {
        code: code.to_string(),
        title: title.to_string(),
        description: "".to_string(),
        port: 0,
        capacity: 0,
        weight: 0.0,
        status: TimelineLedgerStatus::Created,
    }
}
