use super::*;

use acme_kit::text;

/// Where a zone assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ZoneAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ZoneAssignmentStatus {
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

impl std::fmt::Display for ZoneAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a zone assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ZoneAssignment {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub retries: i64,
    pub summary: String,
    pub source: String,
    pub status: ZoneAssignmentStatus,
}

impl ZoneAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ZoneAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ZoneAssignmentStatus) -> bool {
        match self.status {
            ZoneAssignmentStatus::Open => {
                match next {
                    ZoneAssignmentStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAssignmentStatus::InReview => {
                match next {
                    ZoneAssignmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAssignmentStatus::Resolved => {
                match next {
                    ZoneAssignmentStatus::Reopened | ZoneAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ZoneAssignmentStatus::Reopened => {
                return next == ZoneAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ZoneAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "zone assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "zone assignment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> ZoneAssignment {
        ZoneAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            summary: self.summary.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a zone assignment in status open.
pub fn new_zone_assignment(code: &str, title: &str) -> ZoneAssignment {
    ZoneAssignment {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        retries: 0,
        summary: "".to_string(),
        source: "".to_string(),
        status: ZoneAssignmentStatus::Open,
    }
}
