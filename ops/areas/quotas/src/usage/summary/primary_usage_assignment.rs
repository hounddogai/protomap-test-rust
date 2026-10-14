use super::*;

use acme_kit::text;

/// Where a primary usage assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryUsageAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryUsageAssignmentStatus {
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

impl std::fmt::Display for PrimaryUsageAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary usage assignment as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryUsageAssignment {
    pub code: String,
    pub title: String,
    pub namespace: String,
    pub principal: String,
    pub archived: bool,
    pub rate: f64,
    pub reviewer: String,
    pub status: PrimaryUsageAssignmentStatus,
}

impl PrimaryUsageAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryUsageAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryUsageAssignmentStatus) -> bool {
        match self.status {
            PrimaryUsageAssignmentStatus::Draft => {
                match next {
                    PrimaryUsageAssignmentStatus::Suspended | PrimaryUsageAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryUsageAssignmentStatus::Pending => {
                match next {
                    PrimaryUsageAssignmentStatus::Suspended | PrimaryUsageAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryUsageAssignmentStatus::Active => {
                return next == PrimaryUsageAssignmentStatus::Suspended;
            }
            PrimaryUsageAssignmentStatus::Suspended => {
                return next == PrimaryUsageAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryUsageAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary usage assignment {} cannot move from {} to {}",
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
        self.rate
    }

    pub fn copy(&self) -> PrimaryUsageAssignment {
        PrimaryUsageAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            principal: self.principal.clone(),
            archived: self.archived,
            rate: self.rate,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary usage assignment in status draft.
pub fn new_primary_usage_assignment(code: &str, title: &str) -> PrimaryUsageAssignment {
    PrimaryUsageAssignment {
        code: code.to_string(),
        title: title.to_string(),
        namespace: "".to_string(),
        principal: "".to_string(),
        archived: false,
        rate: 0.0,
        reviewer: "".to_string(),
        status: PrimaryUsageAssignmentStatus::Draft,
    }
}
