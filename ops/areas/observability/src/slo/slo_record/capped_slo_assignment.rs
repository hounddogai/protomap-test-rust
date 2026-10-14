use super::*;

use acme_kit::text;

/// Where a capped slo assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedSloAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedSloAssignmentStatus {
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

impl std::fmt::Display for CappedSloAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped slo assignment as the slo team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedSloAssignment {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub created_at: i64,
    pub approved: bool,
    pub ceiling: f64,
    pub status: CappedSloAssignmentStatus,
}

impl CappedSloAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedSloAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedSloAssignmentStatus) -> bool {
        match self.status {
            CappedSloAssignmentStatus::Open => {
                match next {
                    CappedSloAssignmentStatus::InReview | CappedSloAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSloAssignmentStatus::InReview => {
                match next {
                    CappedSloAssignmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedSloAssignmentStatus::Resolved => {
                return next == CappedSloAssignmentStatus::Reopened || next == CappedSloAssignmentStatus::Archived;
            }
            CappedSloAssignmentStatus::Reopened => {
                match next {
                    CappedSloAssignmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedSloAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped slo assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        (self.ceiling * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedSloAssignment {
        CappedSloAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            created_at: self.created_at,
            approved: self.approved,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a capped slo assignment in status open.
pub fn new_capped_slo_assignment(code: &str, title: &str) -> CappedSloAssignment {
    CappedSloAssignment {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        created_at: 0,
        approved: false,
        ceiling: 0.0,
        status: CappedSloAssignmentStatus::Open,
    }
}
