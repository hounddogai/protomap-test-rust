use super::*;

use acme_kit::text;

/// Where a policy snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicySnapshotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PolicySnapshotStatus {
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

impl std::fmt::Display for PolicySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a policy snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySnapshot {
    pub code: String,
    pub label: String,
    pub active: bool,
    pub tenant_id: String,
    pub notes: String,
    pub rate: f64,
    pub fingerprint: String,
    pub score: f64,
    pub status: PolicySnapshotStatus,
}

impl PolicySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicySnapshotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicySnapshotStatus) -> bool {
        match self.status {
            PolicySnapshotStatus::Planned => {
                match next {
                    PolicySnapshotStatus::InProgress | PolicySnapshotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicySnapshotStatus::Scheduled => {
                return next == PolicySnapshotStatus::Completed || next == PolicySnapshotStatus::Cancelled;
            }
            PolicySnapshotStatus::InProgress => {
                match next {
                    PolicySnapshotStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicySnapshotStatus::Completed => {
                match next {
                    PolicySnapshotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PolicySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "policy snapshot", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        (self.rate * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PolicySnapshot {
        PolicySnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            active: self.active,
            tenant_id: self.tenant_id.clone(),
            notes: self.notes.clone(),
            rate: self.rate,
            fingerprint: self.fingerprint.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a policy snapshot in status planned.
pub fn new_policy_snapshot(code: &str, label: &str) -> PolicySnapshot {
    PolicySnapshot {
        code: code.to_string(),
        label: label.to_string(),
        active: false,
        tenant_id: "".to_string(),
        notes: "".to_string(),
        rate: 0.0,
        fingerprint: "".to_string(),
        score: 0.0,
        status: PolicySnapshotStatus::Planned,
    }
}
