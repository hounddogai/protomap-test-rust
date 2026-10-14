use super::*;

use acme_kit::text;

/// Where a allowlist assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AllowlistAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AllowlistAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AllowlistAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One allowlist assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AllowlistAssignment {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub locale: String,
    pub category: String,
    pub segment: String,
    pub status: AllowlistAssignmentStatus,
}

impl AllowlistAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AllowlistAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AllowlistAssignmentStatus) -> bool {
        match self.status {
            AllowlistAssignmentStatus::Requested => {
                return next == AllowlistAssignmentStatus::Granted || next == AllowlistAssignmentStatus::Expired;
            }
            AllowlistAssignmentStatus::Granted => {
                match next {
                    AllowlistAssignmentStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AllowlistAssignmentStatus::Revoked => {
                match next {
                    AllowlistAssignmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: AllowlistAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "allowlist assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.weight
    }

    pub fn copy(&self) -> AllowlistAssignment {
        AllowlistAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            locale: self.locale.clone(),
            category: self.category.clone(),
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a allowlist assignment in status requested.
pub fn new_allowlist_assignment(code: &str, title: &str) -> AllowlistAssignment {
    AllowlistAssignment {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        locale: "".to_string(),
        category: "".to_string(),
        segment: "".to_string(),
        status: AllowlistAssignmentStatus::Requested,
    }
}
