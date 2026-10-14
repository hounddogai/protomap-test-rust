use super::*;

use acme_kit::text;

/// Where a credential assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl CredentialAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for CredentialAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a credential assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAssignment {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub quantity: i64,
    pub ceiling: f64,
    pub external_ref: String,
    pub status: CredentialAssignmentStatus,
}

impl CredentialAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialAssignmentStatus) -> bool {
        match self.status {
            CredentialAssignmentStatus::Planned => {
                return next == CredentialAssignmentStatus::InProgress;
            }
            CredentialAssignmentStatus::Scheduled => {
                return next == CredentialAssignmentStatus::Completed;
            }
            CredentialAssignmentStatus::InProgress => {
                match next {
                    CredentialAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialAssignmentStatus::Completed => {
                match next {
                    CredentialAssignmentStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: CredentialAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential assignment", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.quantity as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialAssignment {
        CredentialAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            quantity: self.quantity,
            ceiling: self.ceiling,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a credential assignment in status planned.
pub fn new_credential_assignment(code: &str, name: &str) -> CredentialAssignment {
    CredentialAssignment {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        quantity: 0,
        ceiling: 0.0,
        external_ref: "".to_string(),
        status: CredentialAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_credential_assignment("code-762", "name-174");
        assert_eq!(item.status, CredentialAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_credential_assignment("code-762", "name-174");
        if item.move_to(CredentialAssignmentStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialAssignmentStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_credential_assignment("code-762", "name-174");
        item.status = CredentialAssignmentStatus::Cancelled;
        assert!(
            item.move_to(CredentialAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
