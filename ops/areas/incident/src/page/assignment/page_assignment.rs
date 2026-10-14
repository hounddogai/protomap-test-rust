use super::*;

use acme_kit::text;

/// Where a page assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PageAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PageAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a page assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PageAssignment {
    pub code: String,
    pub name: String,
    pub notes: String,
    pub rotation_days: i64,
    pub weight: f64,
    pub revision: i64,
    pub status: PageAssignmentStatus,
}

impl PageAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageAssignmentStatus) -> bool {
        match self.status {
            PageAssignmentStatus::Requested => {
                match next {
                    PageAssignmentStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageAssignmentStatus::Granted => {
                return next == PageAssignmentStatus::Revoked || next == PageAssignmentStatus::Expired;
            }
            PageAssignmentStatus::Revoked => {
                match next {
                    PageAssignmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: PageAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.weight + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PageAssignment {
        PageAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            notes: self.notes.clone(),
            rotation_days: self.rotation_days,
            weight: self.weight,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a page assignment in status requested.
pub fn new_page_assignment(code: &str, name: &str) -> PageAssignment {
    PageAssignment {
        code: code.to_string(),
        name: name.to_string(),
        notes: "".to_string(),
        rotation_days: 0,
        weight: 0.0,
        revision: 0,
        status: PageAssignmentStatus::Requested,
    }
}
