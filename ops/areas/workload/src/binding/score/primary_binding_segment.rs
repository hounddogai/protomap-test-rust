use super::*;

use acme_kit::text;

/// Where a primary binding segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBindingSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PrimaryBindingSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for PrimaryBindingSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary binding segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingSegment {
    pub code: String,
    pub label: String,
    pub ratio: f64,
    pub archived: bool,
    pub duration_minutes: i64,
    pub active: bool,
    pub status: PrimaryBindingSegmentStatus,
}

impl PrimaryBindingSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBindingSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBindingSegmentStatus) -> bool {
        match self.status {
            PrimaryBindingSegmentStatus::Open => {
                return next == PrimaryBindingSegmentStatus::Resolved;
            }
            PrimaryBindingSegmentStatus::InReview => {
                match next {
                    PrimaryBindingSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBindingSegmentStatus::Resolved => {
                return next == PrimaryBindingSegmentStatus::Reopened;
            }
            PrimaryBindingSegmentStatus::Reopened => {
                return next == PrimaryBindingSegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryBindingSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary binding segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ratio - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryBindingSegment {
        PrimaryBindingSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            ratio: self.ratio,
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary binding segment in status open.
pub fn new_primary_binding_segment(code: &str, label: &str) -> PrimaryBindingSegment {
    PrimaryBindingSegment {
        code: code.to_string(),
        label: label.to_string(),
        ratio: 0.0,
        archived: false,
        duration_minutes: 0,
        active: false,
        status: PrimaryBindingSegmentStatus::Open,
    }
}
