use super::*;

use acme_kit::text;

/// Where a regional probe window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalProbeWindowStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalProbeWindowStatus {
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

impl std::fmt::Display for RegionalProbeWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional probe window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeWindow {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub active: bool,
    pub sequence: i64,
    pub archived: bool,
    pub status: RegionalProbeWindowStatus,
}

impl RegionalProbeWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalProbeWindowStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalProbeWindowStatus) -> bool {
        match self.status {
            RegionalProbeWindowStatus::Planned => {
                match next {
                    RegionalProbeWindowStatus::Completed | RegionalProbeWindowStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalProbeWindowStatus::Scheduled => {
                match next {
                    RegionalProbeWindowStatus::InProgress | RegionalProbeWindowStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalProbeWindowStatus::InProgress => {
                match next {
                    RegionalProbeWindowStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalProbeWindowStatus::Completed => {
                match next {
                    RegionalProbeWindowStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalProbeWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional probe window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.ratio * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalProbeWindow {
        RegionalProbeWindow {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            active: self.active,
            sequence: self.sequence,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a regional probe window in status planned.
pub fn new_regional_probe_window(code: &str, name: &str) -> RegionalProbeWindow {
    RegionalProbeWindow {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        active: false,
        sequence: 0,
        archived: false,
        status: RegionalProbeWindowStatus::Planned,
    }
}
