use super::*;

use acme_kit::text;

/// Where a action item assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ActionItemAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ActionItemAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ActionItemAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a action item assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ActionItemAssignment {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub grace_days: i64,
    pub visible: bool,
    pub duration_minutes: i64,
    pub margin: f64,
    pub minimum: f64,
    pub checksum: String,
    pub retries: i64,
    pub status: ActionItemAssignmentStatus,
}

impl ActionItemAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ActionItemAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ActionItemAssignmentStatus) -> bool {
        match self.status {
            ActionItemAssignmentStatus::Created => {
                match next {
                    ActionItemAssignmentStatus::Validated | ActionItemAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemAssignmentStatus::Validated => {
                match next {
                    ActionItemAssignmentStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ActionItemAssignmentStatus::Published => {
                return next == ActionItemAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ActionItemAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "action item assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.grace_days as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ActionItemAssignment {
        ActionItemAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            grace_days: self.grace_days,
            visible: self.visible,
            duration_minutes: self.duration_minutes,
            margin: self.margin,
            minimum: self.minimum,
            checksum: self.checksum.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a action item assignment in status created.
pub fn new_action_item_assignment(code: &str, name: &str) -> ActionItemAssignment {
    ActionItemAssignment {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        grace_days: 0,
        visible: false,
        duration_minutes: 0,
        margin: 0.0,
        minimum: 0.0,
        checksum: "".to_string(),
        retries: 0,
        status: ActionItemAssignmentStatus::Created,
    }
}
