use super::*;

use acme_kit::text;

/// Where a postmortem profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemProfileStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PostmortemProfileStatus {
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

impl std::fmt::Display for PostmortemProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemProfile {
    pub code: String,
    pub name: String,
    pub label: String,
    pub capacity: i64,
    pub duration_minutes: i64,
    pub status: PostmortemProfileStatus,
}

impl PostmortemProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemProfileStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemProfileStatus) -> bool {
        match self.status {
            PostmortemProfileStatus::Draft => {
                return next == PostmortemProfileStatus::Suspended || next == PostmortemProfileStatus::Closed;
            }
            PostmortemProfileStatus::Pending => {
                match next {
                    PostmortemProfileStatus::Suspended | PostmortemProfileStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemProfileStatus::Active => {
                return next == PostmortemProfileStatus::Suspended || next == PostmortemProfileStatus::Closed;
            }
            PostmortemProfileStatus::Suspended => {
                return next == PostmortemProfileStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "postmortem profile", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemProfile {
        PostmortemProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            capacity: self.capacity,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a postmortem profile in status draft.
pub fn new_postmortem_profile(code: &str, name: &str) -> PostmortemProfile {
    PostmortemProfile {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        capacity: 0,
        duration_minutes: 0,
        status: PostmortemProfileStatus::Draft,
    }
}
