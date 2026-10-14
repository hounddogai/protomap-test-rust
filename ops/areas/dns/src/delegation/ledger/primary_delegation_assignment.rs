use super::*;

use acme_kit::text;

/// Where a primary delegation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryDelegationAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl PrimaryDelegationAssignmentStatus {
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

impl std::fmt::Display for PrimaryDelegationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary delegation assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryDelegationAssignment {
    pub code: String,
    pub label: String,
    pub cluster: String,
    pub discount: f64,
    pub score: f64,
    pub revision: i64,
    pub status: PrimaryDelegationAssignmentStatus,
}

impl PrimaryDelegationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryDelegationAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryDelegationAssignmentStatus) -> bool {
        match self.status {
            PrimaryDelegationAssignmentStatus::Draft => {
                match next {
                    PrimaryDelegationAssignmentStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDelegationAssignmentStatus::Pending => {
                return next == PrimaryDelegationAssignmentStatus::Suspended
                    || next == PrimaryDelegationAssignmentStatus::Closed;
            }
            PrimaryDelegationAssignmentStatus::Active => {
                match next {
                    PrimaryDelegationAssignmentStatus::Suspended | PrimaryDelegationAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryDelegationAssignmentStatus::Suspended => {
                match next {
                    PrimaryDelegationAssignmentStatus::Closed => {
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

    pub fn move_to(&mut self, next: PrimaryDelegationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary delegation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.discount - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryDelegationAssignment {
        PrimaryDelegationAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            cluster: self.cluster.clone(),
            discount: self.discount,
            score: self.score,
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a primary delegation assignment in status draft.
pub fn new_primary_delegation_assignment(code: &str, label: &str) -> PrimaryDelegationAssignment {
    PrimaryDelegationAssignment {
        code: code.to_string(),
        label: label.to_string(),
        cluster: "".to_string(),
        discount: 0.0,
        score: 0.0,
        revision: 0,
        status: PrimaryDelegationAssignmentStatus::Draft,
    }
}
