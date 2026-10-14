use super::*;

use acme_kit::text;

/// Where a pending freeze assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFreezeAssignmentStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingFreezeAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingFreezeAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending freeze assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFreezeAssignment {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub notes: String,
    pub priority: i64,
    pub created_at: i64,
    pub cluster: String,
    pub grace_days: i64,
    pub title: String,
    pub status: PendingFreezeAssignmentStatus,
}

impl PendingFreezeAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFreezeAssignmentStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFreezeAssignmentStatus) -> bool {
        match self.status {
            PendingFreezeAssignmentStatus::Requested => {
                match next {
                    PendingFreezeAssignmentStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFreezeAssignmentStatus::Granted => {
                return next == PendingFreezeAssignmentStatus::Revoked
                    || next == PendingFreezeAssignmentStatus::Expired;
            }
            PendingFreezeAssignmentStatus::Revoked => {
                match next {
                    PendingFreezeAssignmentStatus::Expired => {
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

    pub fn move_to(&mut self, next: PendingFreezeAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending freeze assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingFreezeAssignment {
        PendingFreezeAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            notes: self.notes.clone(),
            priority: self.priority,
            created_at: self.created_at,
            cluster: self.cluster.clone(),
            grace_days: self.grace_days,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a pending freeze assignment in status requested.
pub fn new_pending_freeze_assignment(code: &str, label: &str) -> PendingFreezeAssignment {
    PendingFreezeAssignment {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        notes: "".to_string(),
        priority: 0,
        created_at: 0,
        cluster: "".to_string(),
        grace_days: 0,
        title: "".to_string(),
        status: PendingFreezeAssignmentStatus::Requested,
    }
}
