use super::*;

use acme_kit::text;

/// Where a runbook window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RunbookWindowStatus {
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

impl std::fmt::Display for RunbookWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A runbook window as the runbook team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookWindow {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub ratio: f64,
    pub audience: String,
    pub country: String,
    pub port: i64,
    pub cost: f64,
    pub retries: i64,
    pub host: String,
    pub status: RunbookWindowStatus,
}

impl RunbookWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookWindowStatus) -> bool {
        match self.status {
            RunbookWindowStatus::Planned => {
                match next {
                    RunbookWindowStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookWindowStatus::Scheduled => {
                return next == RunbookWindowStatus::Completed;
            }
            RunbookWindowStatus::InProgress => {
                return next == RunbookWindowStatus::Completed || next == RunbookWindowStatus::Cancelled;
            }
            RunbookWindowStatus::Completed => {
                match next {
                    RunbookWindowStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RunbookWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.threshold * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookWindow {
        RunbookWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            ratio: self.ratio,
            audience: self.audience.clone(),
            country: self.country.clone(),
            port: self.port,
            cost: self.cost,
            retries: self.retries,
            host: self.host.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a runbook window in status planned.
pub fn new_runbook_window(code: &str, label: &str) -> RunbookWindow {
    RunbookWindow {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        ratio: 0.0,
        audience: "".to_string(),
        country: "".to_string(),
        port: 0,
        cost: 0.0,
        retries: 0,
        host: "".to_string(),
        status: RunbookWindowStatus::Planned,
    }
}
