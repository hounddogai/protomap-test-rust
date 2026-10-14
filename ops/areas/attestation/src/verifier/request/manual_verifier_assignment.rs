use super::*;

use acme_kit::text;

/// Where a manual verifier assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualVerifierAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualVerifierAssignmentStatus {
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

impl std::fmt::Display for ManualVerifierAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual verifier assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualVerifierAssignment {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub timezone: String,
    pub title: String,
    pub batch_size: i64,
    pub minimum: f64,
    pub quantity: i64,
    pub owner_id: String,
    pub status: ManualVerifierAssignmentStatus,
}

impl ManualVerifierAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualVerifierAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualVerifierAssignmentStatus) -> bool {
        match self.status {
            ManualVerifierAssignmentStatus::Idle => {
                match next {
                    ManualVerifierAssignmentStatus::Ready | ManualVerifierAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierAssignmentStatus::Warming => {
                match next {
                    ManualVerifierAssignmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierAssignmentStatus::Ready => {
                match next {
                    ManualVerifierAssignmentStatus::Draining | ManualVerifierAssignmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualVerifierAssignmentStatus::Draining => {
                match next {
                    ManualVerifierAssignmentStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualVerifierAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual verifier assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.minimum + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualVerifierAssignment {
        ManualVerifierAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            minimum: self.minimum,
            quantity: self.quantity,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a manual verifier assignment in status idle.
pub fn new_manual_verifier_assignment(code: &str, name: &str) -> ManualVerifierAssignment {
    ManualVerifierAssignment {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        timezone: "".to_string(),
        title: "".to_string(),
        batch_size: 0,
        minimum: 0.0,
        quantity: 0,
        owner_id: "".to_string(),
        status: ManualVerifierAssignmentStatus::Idle,
    }
}
