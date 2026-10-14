use super::*;

use acme_kit::text;

/// Where a probe window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ProbeWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ProbeWindowStatus {
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

impl std::fmt::Display for ProbeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One probe window, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ProbeWindow {
    pub code: String,
    pub title: String,
    pub audience: String,
    pub issuer: String,
    pub summary: String,
    pub grace_days: i64,
    pub status: ProbeWindowStatus,
}

impl ProbeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ProbeWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ProbeWindowStatus) -> bool {
        match self.status {
            ProbeWindowStatus::Planned => {
                return next == ProbeWindowStatus::Scheduled || next == ProbeWindowStatus::Completed;
            }
            ProbeWindowStatus::Scheduled => {
                return next == ProbeWindowStatus::Completed || next == ProbeWindowStatus::Cancelled;
            }
            ProbeWindowStatus::InProgress => {
                match next {
                    ProbeWindowStatus::Completed | ProbeWindowStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ProbeWindowStatus::Completed => {
                match next {
                    ProbeWindowStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: ProbeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "probe window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> ProbeWindow {
        ProbeWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            audience: self.audience.clone(),
            issuer: self.issuer.clone(),
            summary: self.summary.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a probe window in status planned.
pub fn new_probe_window(code: &str, title: &str) -> ProbeWindow {
    ProbeWindow {
        code: code.to_string(),
        title: title.to_string(),
        audience: "".to_string(),
        issuer: "".to_string(),
        summary: "".to_string(),
        grace_days: 0,
        status: ProbeWindowStatus::Planned,
    }
}
