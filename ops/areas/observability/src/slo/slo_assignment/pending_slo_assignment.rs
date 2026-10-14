use super::*;

use acme_kit::text;

/// Where a pending slo assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSloAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingSloAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingSloAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One pending slo assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSloAssignment {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub approved: bool,
    pub batch_size: i64,
    pub status: PendingSloAssignmentStatus,
}

impl PendingSloAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSloAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSloAssignmentStatus) -> bool {
        match self.status {
            PendingSloAssignmentStatus::Requested => {
                return next == PendingSloAssignmentStatus::Revoked || next == PendingSloAssignmentStatus::Expired;
            }
            PendingSloAssignmentStatus::Granted => {
                return next == PendingSloAssignmentStatus::Revoked || next == PendingSloAssignmentStatus::Expired;
            }
            PendingSloAssignmentStatus::Revoked => {
                match next {
                    PendingSloAssignmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingSloAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending slo assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.batch_size as f64
    }

    pub fn copy(&self) -> PendingSloAssignment {
        PendingSloAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            approved: self.approved,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending slo assignment in status requested.
pub fn new_pending_slo_assignment(code: &str, name: &str) -> PendingSloAssignment {
    PendingSloAssignment {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        approved: false,
        batch_size: 0,
        status: PendingSloAssignmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_pending_slo_assignment("code-445", "name-865");
        assert_eq!(item.status, PendingSloAssignmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_pending_slo_assignment("code-445", "name-865");
        if item.move_to(PendingSloAssignmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingSloAssignmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_pending_slo_assignment("code-445", "name-865");
        item.status = PendingSloAssignmentStatus::Expired;
        assert!(
            item.move_to(PendingSloAssignmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
