use super::*;

use acme_kit::text;

/// Where a suspension target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SuspensionTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SuspensionTargetStatus {
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

impl std::fmt::Display for SuspensionTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One suspension target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SuspensionTarget {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub margin: f64,
    pub timezone: String,
    pub weight: f64,
    pub namespace: String,
    pub capacity: i64,
    pub category: String,
    pub scope: String,
    pub status: SuspensionTargetStatus,
}

impl SuspensionTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SuspensionTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SuspensionTargetStatus) -> bool {
        match self.status {
            SuspensionTargetStatus::Planned => {
                match next {
                    SuspensionTargetStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SuspensionTargetStatus::Scheduled => {
                return next == SuspensionTargetStatus::Cancelled;
            }
            SuspensionTargetStatus::InProgress => {
                return next == SuspensionTargetStatus::Cancelled;
            }
            SuspensionTargetStatus::Completed => {
                return next == SuspensionTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SuspensionTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "suspension target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_load(&self) -> f64 {
        (self.weight * self.margin * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SuspensionTarget {
        SuspensionTarget {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            margin: self.margin,
            timezone: self.timezone.clone(),
            weight: self.weight,
            namespace: self.namespace.clone(),
            capacity: self.capacity,
            category: self.category.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a suspension target in status planned.
pub fn new_suspension_target(code: &str, label: &str) -> SuspensionTarget {
    SuspensionTarget {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        margin: 0.0,
        timezone: "".to_string(),
        weight: 0.0,
        namespace: "".to_string(),
        capacity: 0,
        category: "".to_string(),
        scope: "".to_string(),
        status: SuspensionTargetStatus::Planned,
    }
}
