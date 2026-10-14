use super::*;

use acme_kit::text;

/// Where a global release assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleaseAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GlobalReleaseAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GlobalReleaseAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global release assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseAssignment {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub active: bool,
    pub currency: String,
    pub principal: String,
    pub burst: i64,
    pub title: String,
    pub contact_email: String,
    pub ceiling: f64,
    pub score: f64,
    pub status: GlobalReleaseAssignmentStatus,
}

impl GlobalReleaseAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleaseAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleaseAssignmentStatus) -> bool {
        match self.status {
            GlobalReleaseAssignmentStatus::Proposed => {
                return next == GlobalReleaseAssignmentStatus::Approved
                    || next == GlobalReleaseAssignmentStatus::Rejected;
            }
            GlobalReleaseAssignmentStatus::Approved => {
                match next {
                    GlobalReleaseAssignmentStatus::Rejected | GlobalReleaseAssignmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseAssignmentStatus::Rejected => {
                return next == GlobalReleaseAssignmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalReleaseAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ceiling + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReleaseAssignment {
        GlobalReleaseAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            active: self.active,
            currency: self.currency.clone(),
            principal: self.principal.clone(),
            burst: self.burst,
            title: self.title.clone(),
            contact_email: self.contact_email.clone(),
            ceiling: self.ceiling,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a global release assignment in status proposed.
pub fn new_global_release_assignment(code: &str, name: &str) -> GlobalReleaseAssignment {
    GlobalReleaseAssignment {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        active: false,
        currency: "".to_string(),
        principal: "".to_string(),
        burst: 0,
        title: "".to_string(),
        contact_email: "".to_string(),
        ceiling: 0.0,
        score: 0.0,
        status: GlobalReleaseAssignmentStatus::Proposed,
    }
}
