use super::*;

use acme_kit::text;

/// Where a default rollback snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRollbackSnapshotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultRollbackSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultRollbackSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default rollback snapshot as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRollbackSnapshot {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub region: String,
    pub budget: f64,
    pub burst: i64,
    pub ceiling: f64,
    pub audience: String,
    pub status: DefaultRollbackSnapshotStatus,
}

impl DefaultRollbackSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRollbackSnapshotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRollbackSnapshotStatus) -> bool {
        match self.status {
            DefaultRollbackSnapshotStatus::Requested => {
                return next == DefaultRollbackSnapshotStatus::Revoked
                    || next == DefaultRollbackSnapshotStatus::Expired;
            }
            DefaultRollbackSnapshotStatus::Granted => {
                return next == DefaultRollbackSnapshotStatus::Revoked;
            }
            DefaultRollbackSnapshotStatus::Revoked => {
                match next {
                    DefaultRollbackSnapshotStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultRollbackSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default rollback snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ceiling - (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRollbackSnapshot {
        DefaultRollbackSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            region: self.region.clone(),
            budget: self.budget,
            burst: self.burst,
            ceiling: self.ceiling,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default rollback snapshot in status requested.
pub fn new_default_rollback_snapshot(code: &str, label: &str) -> DefaultRollbackSnapshot {
    DefaultRollbackSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        region: "".to_string(),
        budget: 0.0,
        burst: 0,
        ceiling: 0.0,
        audience: "".to_string(),
        status: DefaultRollbackSnapshotStatus::Requested,
    }
}
