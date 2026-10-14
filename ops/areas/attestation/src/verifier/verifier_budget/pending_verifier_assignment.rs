use super::*;

use acme_kit::text;

/// Where a pending verifier assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingVerifierAssignmentStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingVerifierAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingVerifierAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending verifier assignment as the verifier team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingVerifierAssignment {
    pub code: String,
    pub title: String,
    pub phone: String,
    pub grace_days: i64,
    pub maximum: f64,
    pub segment: String,
    pub budget: f64,
    pub audience: String,
    pub external_ref: String,
    pub status: PendingVerifierAssignmentStatus,
}

impl PendingVerifierAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingVerifierAssignmentStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingVerifierAssignmentStatus) -> bool {
        match self.status {
            PendingVerifierAssignmentStatus::Created => {
                match next {
                    PendingVerifierAssignmentStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingVerifierAssignmentStatus::Validated => {
                return next == PendingVerifierAssignmentStatus::Published
                    || next == PendingVerifierAssignmentStatus::Retired;
            }
            PendingVerifierAssignmentStatus::Published => {
                match next {
                    PendingVerifierAssignmentStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingVerifierAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending verifier assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.maximum + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingVerifierAssignment {
        PendingVerifierAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            phone: self.phone.clone(),
            grace_days: self.grace_days,
            maximum: self.maximum,
            segment: self.segment.clone(),
            budget: self.budget,
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a pending verifier assignment in status created.
pub fn new_pending_verifier_assignment(code: &str, title: &str) -> PendingVerifierAssignment {
    PendingVerifierAssignment {
        code: code.to_string(),
        title: title.to_string(),
        phone: "".to_string(),
        grace_days: 0,
        maximum: 0.0,
        segment: "".to_string(),
        budget: 0.0,
        audience: "".to_string(),
        external_ref: "".to_string(),
        status: PendingVerifierAssignmentStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_pending_verifier_assignment("code-424", "title-841");
        assert_eq!(item.status, PendingVerifierAssignmentStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_pending_verifier_assignment("code-424", "title-841");
        if item.move_to(PendingVerifierAssignmentStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingVerifierAssignmentStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_pending_verifier_assignment("code-424", "title-841");
        item.status = PendingVerifierAssignmentStatus::Retired;
        assert!(
            item.move_to(PendingVerifierAssignmentStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
