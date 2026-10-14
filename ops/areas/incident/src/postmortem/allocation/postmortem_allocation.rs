use super::*;

use acme_kit::text;

/// Where a postmortem allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemAllocationStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PostmortemAllocationStatus {
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

impl std::fmt::Display for PostmortemAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemAllocation {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub maximum: f64,
    pub grace_days: i64,
    pub status: PostmortemAllocationStatus,
}

impl PostmortemAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemAllocationStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemAllocationStatus) -> bool {
        match self.status {
            PostmortemAllocationStatus::Planned => {
                return next == PostmortemAllocationStatus::Completed;
            }
            PostmortemAllocationStatus::Scheduled => {
                return next == PostmortemAllocationStatus::Cancelled;
            }
            PostmortemAllocationStatus::InProgress => {
                return next == PostmortemAllocationStatus::Completed;
            }
            PostmortemAllocationStatus::Completed => {
                match next {
                    PostmortemAllocationStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PostmortemAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.grace_days as f64 + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemAllocation {
        PostmortemAllocation {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            maximum: self.maximum,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a postmortem allocation in status planned.
pub fn new_postmortem_allocation(code: &str, title: &str) -> PostmortemAllocation {
    PostmortemAllocation {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        maximum: 0.0,
        grace_days: 0,
        status: PostmortemAllocationStatus::Planned,
    }
}
