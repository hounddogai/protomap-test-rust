use super::*;

use acme_kit::text;

/// Where a release assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseAssignmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ReleaseAssignmentStatus {
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

impl std::fmt::Display for ReleaseAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release assignment as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseAssignment {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub description: String,
    pub maximum: f64,
    pub archived: bool,
    pub status: ReleaseAssignmentStatus,
}

impl ReleaseAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseAssignmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseAssignmentStatus) -> bool {
        match self.status {
            ReleaseAssignmentStatus::Idle => {
                return next == ReleaseAssignmentStatus::Stopped;
            }
            ReleaseAssignmentStatus::Warming => {
                match next {
                    ReleaseAssignmentStatus::Ready | ReleaseAssignmentStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseAssignmentStatus::Ready => {
                return next == ReleaseAssignmentStatus::Stopped;
            }
            ReleaseAssignmentStatus::Draining => {
                return next == ReleaseAssignmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release assignment {} cannot move from {} to {}",
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
        self.maximum
    }

    pub fn copy(&self) -> ReleaseAssignment {
        ReleaseAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            description: self.description.clone(),
            maximum: self.maximum,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release assignment in status idle.
pub fn new_release_assignment(code: &str, name: &str) -> ReleaseAssignment {
    ReleaseAssignment {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        description: "".to_string(),
        maximum: 0.0,
        archived: false,
        status: ReleaseAssignmentStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_release_assignment("code-615", "name-755");
        assert_eq!(item.status, ReleaseAssignmentStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_release_assignment("code-615", "name-755");
        if item.move_to(ReleaseAssignmentStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseAssignmentStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_release_assignment("code-615", "name-755");
        item.status = ReleaseAssignmentStatus::Stopped;
        assert!(
            item.move_to(ReleaseAssignmentStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
