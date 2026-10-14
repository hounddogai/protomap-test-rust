use super::*;

use acme_kit::text;

/// Where a verifier target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum VerifierTargetStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl VerifierTargetStatus {
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

impl std::fmt::Display for VerifierTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One verifier target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct VerifierTarget {
    pub code: String,
    pub name: String,
    pub threshold: f64,
    pub unit: String,
    pub description: String,
    pub rate: f64,
    pub status: VerifierTargetStatus,
}

impl VerifierTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            VerifierTargetStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: VerifierTargetStatus) -> bool {
        match self.status {
            VerifierTargetStatus::Planned => {
                match next {
                    VerifierTargetStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierTargetStatus::Scheduled => {
                match next {
                    VerifierTargetStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            VerifierTargetStatus::InProgress => {
                return next == VerifierTargetStatus::Completed;
            }
            VerifierTargetStatus::Completed => {
                return next == VerifierTargetStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: VerifierTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "verifier target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.rate + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> VerifierTarget {
        VerifierTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            threshold: self.threshold,
            unit: self.unit.clone(),
            description: self.description.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a verifier target in status planned.
pub fn new_verifier_target(code: &str, name: &str) -> VerifierTarget {
    VerifierTarget {
        code: code.to_string(),
        name: name.to_string(),
        threshold: 0.0,
        unit: "".to_string(),
        description: "".to_string(),
        rate: 0.0,
        status: VerifierTargetStatus::Planned,
    }
}
