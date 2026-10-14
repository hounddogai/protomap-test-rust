use super::*;

use acme_kit::text;

/// Where a escalation assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationAssignmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EscalationAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EscalationAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One escalation assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationAssignment {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub port: i64,
    pub pinned: bool,
    pub threshold: f64,
    pub cipher: String,
    pub sequence: i64,
    pub checksum: String,
    pub summary: String,
    pub grace_days: i64,
    pub status: EscalationAssignmentStatus,
}

impl EscalationAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationAssignmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationAssignmentStatus) -> bool {
        match self.status {
            EscalationAssignmentStatus::Proposed => {
                match next {
                    EscalationAssignmentStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAssignmentStatus::Approved => {
                match next {
                    EscalationAssignmentStatus::Rejected | EscalationAssignmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationAssignmentStatus::Rejected => {
                match next {
                    EscalationAssignmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: EscalationAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.sequence as f64) * (self.port as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationAssignment {
        EscalationAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            port: self.port,
            pinned: self.pinned,
            threshold: self.threshold,
            cipher: self.cipher.clone(),
            sequence: self.sequence,
            checksum: self.checksum.clone(),
            summary: self.summary.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a escalation assignment in status proposed.
pub fn new_escalation_assignment(code: &str, title: &str) -> EscalationAssignment {
    EscalationAssignment {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        port: 0,
        pinned: false,
        threshold: 0.0,
        cipher: "".to_string(),
        sequence: 0,
        checksum: "".to_string(),
        summary: "".to_string(),
        grace_days: 0,
        status: EscalationAssignmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_escalation_assignment("code-398", "title-873");
        assert_eq!(item.status, EscalationAssignmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_escalation_assignment("code-398", "title-873");
        if item.move_to(EscalationAssignmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationAssignmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_escalation_assignment("code-398", "title-873");
        item.status = EscalationAssignmentStatus::Withdrawn;
        assert!(
            item.move_to(EscalationAssignmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
