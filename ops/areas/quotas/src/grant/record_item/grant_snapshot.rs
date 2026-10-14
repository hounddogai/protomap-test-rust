use super::*;

use acme_kit::text;

/// Where a grant snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GrantSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl GrantSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for GrantSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a grant snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GrantSnapshot {
    pub code: String,
    pub title: String,
    pub expires_at: i64,
    pub active: bool,
    pub attempts: i64,
    pub unit: String,
    pub revision: i64,
    pub status: GrantSnapshotStatus,
}

impl GrantSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GrantSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GrantSnapshotStatus) -> bool {
        match self.status {
            GrantSnapshotStatus::Proposed => {
                match next {
                    GrantSnapshotStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantSnapshotStatus::Approved => {
                match next {
                    GrantSnapshotStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GrantSnapshotStatus::Rejected => {
                match next {
                    GrantSnapshotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: GrantSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "grant snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GrantSnapshot {
        GrantSnapshot {
            code: self.code.clone(),
            title: self.title.clone(),
            expires_at: self.expires_at,
            active: self.active,
            attempts: self.attempts,
            unit: self.unit.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a grant snapshot in status proposed.
pub fn new_grant_snapshot(code: &str, title: &str) -> GrantSnapshot {
    GrantSnapshot {
        code: code.to_string(),
        title: title.to_string(),
        expires_at: 0,
        active: false,
        attempts: 0,
        unit: "".to_string(),
        revision: 0,
        status: GrantSnapshotStatus::Proposed,
    }
}
