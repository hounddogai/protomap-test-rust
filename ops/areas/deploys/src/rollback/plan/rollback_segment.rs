use super::*;

use acme_kit::text;

/// Where a rollback segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollbackSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollbackSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback segment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSegment {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub budget: f64,
    pub fingerprint: String,
    pub currency: String,
    pub status: RollbackSegmentStatus,
}

impl RollbackSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackSegmentStatus) -> bool {
        match self.status {
            RollbackSegmentStatus::Proposed => {
                match next {
                    RollbackSegmentStatus::Rejected | RollbackSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSegmentStatus::Approved => {
                match next {
                    RollbackSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSegmentStatus::Rejected => {
                match next {
                    RollbackSegmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollbackSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.budget
    }

    pub fn copy(&self) -> RollbackSegment {
        RollbackSegment {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            budget: self.budget,
            fingerprint: self.fingerprint.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a rollback segment in status proposed.
pub fn new_rollback_segment(code: &str, name: &str) -> RollbackSegment {
    RollbackSegment {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        budget: 0.0,
        fingerprint: "".to_string(),
        currency: "".to_string(),
        status: RollbackSegmentStatus::Proposed,
    }
}
