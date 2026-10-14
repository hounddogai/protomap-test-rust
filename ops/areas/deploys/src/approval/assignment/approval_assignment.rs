use super::*;

use acme_kit::text;

/// Where a approval assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ApprovalAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ApprovalAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ApprovalAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One approval assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ApprovalAssignment {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub maximum: f64,
    pub checksum: String,
    pub audience: String,
    pub external_ref: String,
    pub source: String,
    pub cost: f64,
    pub duration_minutes: i64,
    pub status: ApprovalAssignmentStatus,
}

impl ApprovalAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ApprovalAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ApprovalAssignmentStatus) -> bool {
        match self.status {
            ApprovalAssignmentStatus::Idle => {
                return next == ApprovalAssignmentStatus::Ready || next == ApprovalAssignmentStatus::Stopped;
            }
            ApprovalAssignmentStatus::Warming => {
                return next == ApprovalAssignmentStatus::Draining;
            }
            ApprovalAssignmentStatus::Ready => {
                match next {
                    ApprovalAssignmentStatus::Draining | ApprovalAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ApprovalAssignmentStatus::Draining => {
                match next {
                    ApprovalAssignmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ApprovalAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "approval assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "approval assignment", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ApprovalAssignment {
        ApprovalAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            source: self.source.clone(),
            cost: self.cost,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a approval assignment in status idle.
pub fn new_approval_assignment(code: &str, label: &str) -> ApprovalAssignment {
    ApprovalAssignment {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        maximum: 0.0,
        checksum: "".to_string(),
        audience: "".to_string(),
        external_ref: "".to_string(),
        source: "".to_string(),
        cost: 0.0,
        duration_minutes: 0,
        status: ApprovalAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_approval_assignment("code-247", "label-210");
        assert_eq!(item.status, ApprovalAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_approval_assignment("code-247", "label-210");
        if item.move_to(ApprovalAssignmentStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ApprovalAssignmentStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_approval_assignment("code-247", "label-210");
        item.status = ApprovalAssignmentStatus::Stopped;
        assert!(
            item.move_to(ApprovalAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
