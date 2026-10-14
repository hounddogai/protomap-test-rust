use super::*;

use acme_kit::text;

/// Where a canary checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryCheckpointStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CanaryCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CanaryCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryCheckpoint {
    pub code: String,
    pub name: String,
    pub owner_id: String,
    pub rotation_days: i64,
    pub active: bool,
    pub duration_minutes: i64,
    pub contact_email: String,
    pub country: String,
    pub status: CanaryCheckpointStatus,
}

impl CanaryCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryCheckpointStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryCheckpointStatus) -> bool {
        match self.status {
            CanaryCheckpointStatus::Proposed => {
                match next {
                    CanaryCheckpointStatus::Approved | CanaryCheckpointStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryCheckpointStatus::Approved => {
                return next == CanaryCheckpointStatus::Rejected || next == CanaryCheckpointStatus::Withdrawn;
            }
            CanaryCheckpointStatus::Rejected => {
                match next {
                    CanaryCheckpointStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: CanaryCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.rotation_days as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryCheckpoint {
        CanaryCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            owner_id: self.owner_id.clone(),
            rotation_days: self.rotation_days,
            active: self.active,
            duration_minutes: self.duration_minutes,
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a canary checkpoint in status proposed.
pub fn new_canary_checkpoint(code: &str, name: &str) -> CanaryCheckpoint {
    CanaryCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        owner_id: "".to_string(),
        rotation_days: 0,
        active: false,
        duration_minutes: 0,
        contact_email: "".to_string(),
        country: "".to_string(),
        status: CanaryCheckpointStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_canary_checkpoint("code-946", "name-731");
        assert_eq!(item.status, CanaryCheckpointStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_canary_checkpoint("code-946", "name-731");
        if item.move_to(CanaryCheckpointStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryCheckpointStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_canary_checkpoint("code-946", "name-731");
        item.status = CanaryCheckpointStatus::Withdrawn;
        assert!(
            item.move_to(CanaryCheckpointStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
