use super::*;

use acme_kit::text;

/// Where a rollout assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RolloutAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RolloutAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollout assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutAssignment {
    pub code: String,
    pub label: String,
    pub quantity: i64,
    pub source: String,
    pub channel: String,
    pub namespace: String,
    pub audience: String,
    pub external_ref: String,
    pub notes: String,
    pub description: String,
    pub duration_minutes: i64,
    pub status: RolloutAssignmentStatus,
}

impl RolloutAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutAssignmentStatus) -> bool {
        match self.status {
            RolloutAssignmentStatus::Requested => {
                return next == RolloutAssignmentStatus::Granted || next == RolloutAssignmentStatus::Expired;
            }
            RolloutAssignmentStatus::Granted => {
                match next {
                    RolloutAssignmentStatus::Revoked | RolloutAssignmentStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutAssignmentStatus::Revoked => {
                return next == RolloutAssignmentStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.duration_minutes as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutAssignment {
        RolloutAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            quantity: self.quantity,
            source: self.source.clone(),
            channel: self.channel.clone(),
            namespace: self.namespace.clone(),
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            notes: self.notes.clone(),
            description: self.description.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollout assignment in status requested.
pub fn new_rollout_assignment(code: &str, label: &str) -> RolloutAssignment {
    RolloutAssignment {
        code: code.to_string(),
        label: label.to_string(),
        quantity: 0,
        source: "".to_string(),
        channel: "".to_string(),
        namespace: "".to_string(),
        audience: "".to_string(),
        external_ref: "".to_string(),
        notes: "".to_string(),
        description: "".to_string(),
        duration_minutes: 0,
        status: RolloutAssignmentStatus::Requested,
    }
}
