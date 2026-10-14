use super::*;

use acme_kit::text;

/// Where a slo ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SloLedgerStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SloLedgerStatus {
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

impl std::fmt::Display for SloLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One slo ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SloLedger {
    pub code: String,
    pub name: String,
    pub priority: i64,
    pub source: String,
    pub limit: i64,
    pub batch_size: i64,
    pub budget: f64,
    pub category: String,
    pub weight: f64,
    pub duration_minutes: i64,
    pub status: SloLedgerStatus,
}

impl SloLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SloLedgerStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SloLedgerStatus) -> bool {
        match self.status {
            SloLedgerStatus::Planned => {
                return next == SloLedgerStatus::Scheduled;
            }
            SloLedgerStatus::Scheduled => {
                return next == SloLedgerStatus::InProgress;
            }
            SloLedgerStatus::InProgress => {
                match next {
                    SloLedgerStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SloLedgerStatus::Completed => {
                return next == SloLedgerStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SloLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "slo ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        (self.budget * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SloLedger {
        SloLedger {
            code: self.code.clone(),
            name: self.name.clone(),
            priority: self.priority,
            source: self.source.clone(),
            limit: self.limit,
            batch_size: self.batch_size,
            budget: self.budget,
            category: self.category.clone(),
            weight: self.weight,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a slo ledger in status planned.
pub fn new_slo_ledger(code: &str, name: &str) -> SloLedger {
    SloLedger {
        code: code.to_string(),
        name: name.to_string(),
        priority: 0,
        source: "".to_string(),
        limit: 0,
        batch_size: 0,
        budget: 0.0,
        category: "".to_string(),
        weight: 0.0,
        duration_minutes: 0,
        status: SloLedgerStatus::Planned,
    }
}
