use super::*;

use acme_kit::text;

/// Where a peering assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PeeringAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PeeringAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A peering assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringAssignment {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub namespace: String,
    pub channel: String,
    pub checksum: String,
    pub attempts: i64,
    pub status: PeeringAssignmentStatus,
}

impl PeeringAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringAssignmentStatus) -> bool {
        match self.status {
            PeeringAssignmentStatus::Requested => {
                return next == PeeringAssignmentStatus::Revoked;
            }
            PeeringAssignmentStatus::Granted => {
                match next {
                    PeeringAssignmentStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PeeringAssignmentStatus::Revoked => {
                return next == PeeringAssignmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "peering assignment", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> PeeringAssignment {
        PeeringAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            namespace: self.namespace.clone(),
            channel: self.channel.clone(),
            checksum: self.checksum.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a peering assignment in status requested.
pub fn new_peering_assignment(code: &str, title: &str) -> PeeringAssignment {
    PeeringAssignment {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        namespace: "".to_string(),
        channel: "".to_string(),
        checksum: "".to_string(),
        attempts: 0,
        status: PeeringAssignmentStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_peering_assignment("code-272", "title-137");
        assert_eq!(item.status, PeeringAssignmentStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_peering_assignment("code-272", "title-137");
        if item.move_to(PeeringAssignmentStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PeeringAssignmentStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_peering_assignment("code-272", "title-137");
        item.status = PeeringAssignmentStatus::Expired;
        assert!(
            item.move_to(PeeringAssignmentStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
