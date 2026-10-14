use super::*;

use acme_kit::text;

/// Where a canary assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl CanaryAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for CanaryAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One canary assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAssignment {
    pub code: String,
    pub label: String,
    pub batch_size: i64,
    pub cost: f64,
    pub unit: String,
    pub status: CanaryAssignmentStatus,
}

impl CanaryAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryAssignmentStatus) -> bool {
        match self.status {
            CanaryAssignmentStatus::Created => {
                match next {
                    CanaryAssignmentStatus::Validated | CanaryAssignmentStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAssignmentStatus::Validated => {
                return next == CanaryAssignmentStatus::Retired;
            }
            CanaryAssignmentStatus::Published => {
                return next == CanaryAssignmentStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.cost * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryAssignment {
        CanaryAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            batch_size: self.batch_size,
            cost: self.cost,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a canary assignment in status created.
pub fn new_canary_assignment(code: &str, label: &str) -> CanaryAssignment {
    CanaryAssignment {
        code: code.to_string(),
        label: label.to_string(),
        batch_size: 0,
        cost: 0.0,
        unit: "".to_string(),
        status: CanaryAssignmentStatus::Created,
    }
}
