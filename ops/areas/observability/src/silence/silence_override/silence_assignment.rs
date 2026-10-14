use super::*;

use acme_kit::text;

/// Where a silence assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SilenceAssignmentStatus {
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

impl std::fmt::Display for SilenceAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceAssignment {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub approved: bool,
    pub channel: String,
    pub region: String,
    pub cost: f64,
    pub rate: f64,
    pub status: SilenceAssignmentStatus,
}

impl SilenceAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceAssignmentStatus) -> bool {
        match self.status {
            SilenceAssignmentStatus::Open => {
                return next == SilenceAssignmentStatus::InReview || next == SilenceAssignmentStatus::Archived;
            }
            SilenceAssignmentStatus::InReview => {
                match next {
                    SilenceAssignmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceAssignmentStatus::Resolved => {
                return next == SilenceAssignmentStatus::Reopened;
            }
            SilenceAssignmentStatus::Reopened => {
                return next == SilenceAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence assignment", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.rate + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SilenceAssignment {
        SilenceAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            approved: self.approved,
            channel: self.channel.clone(),
            region: self.region.clone(),
            cost: self.cost,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a silence assignment in status open.
pub fn new_silence_assignment(code: &str, name: &str) -> SilenceAssignment {
    SilenceAssignment {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        approved: false,
        channel: "".to_string(),
        region: "".to_string(),
        cost: 0.0,
        rate: 0.0,
        status: SilenceAssignmentStatus::Open,
    }
}
