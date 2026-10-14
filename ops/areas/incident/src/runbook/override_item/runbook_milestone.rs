use super::*;

use acme_kit::text;

/// Where a runbook milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RunbookMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RunbookMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RunbookMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a runbook milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RunbookMilestone {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub audience: String,
    pub discount: f64,
    pub created_at: i64,
    pub source: String,
    pub status: RunbookMilestoneStatus,
}

impl RunbookMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RunbookMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RunbookMilestoneStatus) -> bool {
        match self.status {
            RunbookMilestoneStatus::Draft => {
                return next == RunbookMilestoneStatus::Active;
            }
            RunbookMilestoneStatus::Pending => {
                match next {
                    RunbookMilestoneStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RunbookMilestoneStatus::Active => {
                return next == RunbookMilestoneStatus::Closed;
            }
            RunbookMilestoneStatus::Suspended => {
                return next == RunbookMilestoneStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RunbookMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "runbook milestone {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "runbook milestone", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.ends_at as f64) * self.discount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RunbookMilestone {
        RunbookMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            audience: self.audience.clone(),
            discount: self.discount,
            created_at: self.created_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a runbook milestone in status draft.
pub fn new_runbook_milestone(code: &str, name: &str) -> RunbookMilestone {
    RunbookMilestone {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        audience: "".to_string(),
        discount: 0.0,
        created_at: 0,
        source: "".to_string(),
        status: RunbookMilestoneStatus::Draft,
    }
}
