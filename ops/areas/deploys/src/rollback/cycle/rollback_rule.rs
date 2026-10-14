use super::*;

use acme_kit::text;

/// Where a rollback rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollbackRuleStatus {
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

impl std::fmt::Display for RollbackRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackRule {
    pub code: String,
    pub label: String,
    pub phone: String,
    pub sequence: i64,
    pub rotation_days: i64,
    pub status: RollbackRuleStatus,
}

impl RollbackRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackRuleStatus) -> bool {
        match self.status {
            RollbackRuleStatus::Planned => {
                return next == RollbackRuleStatus::Completed;
            }
            RollbackRuleStatus::Scheduled => {
                match next {
                    RollbackRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackRuleStatus::InProgress => {
                match next {
                    RollbackRuleStatus::Completed | RollbackRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackRuleStatus::Completed => {
                return next == RollbackRuleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rollback rule", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.sequence as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackRule {
        RollbackRule {
            code: self.code.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            sequence: self.sequence,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rollback rule in status planned.
pub fn new_rollback_rule(code: &str, label: &str) -> RollbackRule {
    RollbackRule {
        code: code.to_string(),
        label: label.to_string(),
        phone: "".to_string(),
        sequence: 0,
        rotation_days: 0,
        status: RollbackRuleStatus::Planned,
    }
}
