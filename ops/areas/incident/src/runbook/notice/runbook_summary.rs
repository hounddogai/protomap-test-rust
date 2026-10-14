use super::*;

use acme_kit::text;

/// Where a runbook summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RunbookSummaryStatus {
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

impl std::fmt::Display for RunbookSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One runbook summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookSummary {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub unit: String,
    pub updated_at: i64,
    pub approved: bool,
    pub category: String,
    pub active: bool,
    pub status: RunbookSummaryStatus,
}

impl RunbookSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookSummaryStatus) -> bool {
        match self.status {
            RunbookSummaryStatus::Planned => {
                return next == RunbookSummaryStatus::Scheduled || next == RunbookSummaryStatus::Completed;
            }
            RunbookSummaryStatus::Scheduled => {
                match next {
                    RunbookSummaryStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookSummaryStatus::InProgress => {
                return next == RunbookSummaryStatus::Cancelled;
            }
            RunbookSummaryStatus::Completed => {
                match next {
                    RunbookSummaryStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RunbookSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook summary {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookSummary {
        RunbookSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            unit: self.unit.clone(),
            updated_at: self.updated_at,
            approved: self.approved,
            category: self.category.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a runbook summary in status planned.
pub fn new_runbook_summary(code: &str, name: &str) -> RunbookSummary {
    RunbookSummary {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        unit: "".to_string(),
        updated_at: 0,
        approved: false,
        category: "".to_string(),
        active: false,
        status: RunbookSummaryStatus::Planned,
    }
}
