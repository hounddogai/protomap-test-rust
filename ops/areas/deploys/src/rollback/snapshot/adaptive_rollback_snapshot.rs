use super::*;

use acme_kit::text;

/// Where a adaptive rollback snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRollbackSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl AdaptiveRollbackSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for AdaptiveRollbackSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive rollback snapshot as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRollbackSnapshot {
    pub code: String,
    pub name: String,
    pub cipher: String,
    pub window_days: i64,
    pub host: String,
    pub weight: f64,
    pub reviewer: String,
    pub duration_minutes: i64,
    pub discount: f64,
    pub margin: f64,
    pub category: String,
    pub status: AdaptiveRollbackSnapshotStatus,
}

impl AdaptiveRollbackSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRollbackSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRollbackSnapshotStatus) -> bool {
        match self.status {
            AdaptiveRollbackSnapshotStatus::Open => {
                return next == AdaptiveRollbackSnapshotStatus::Resolved
                    || next == AdaptiveRollbackSnapshotStatus::Archived;
            }
            AdaptiveRollbackSnapshotStatus::InReview => {
                match next {
                    AdaptiveRollbackSnapshotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveRollbackSnapshotStatus::Resolved => {
                return next == AdaptiveRollbackSnapshotStatus::Archived;
            }
            AdaptiveRollbackSnapshotStatus::Reopened => {
                match next {
                    AdaptiveRollbackSnapshotStatus::Archived => {
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

    pub fn move_to(&mut self, next: AdaptiveRollbackSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rollback snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        (self.discount * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRollbackSnapshot {
        AdaptiveRollbackSnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            cipher: self.cipher.clone(),
            window_days: self.window_days,
            host: self.host.clone(),
            weight: self.weight,
            reviewer: self.reviewer.clone(),
            duration_minutes: self.duration_minutes,
            discount: self.discount,
            margin: self.margin,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a adaptive rollback snapshot in status open.
pub fn new_adaptive_rollback_snapshot(code: &str, name: &str) -> AdaptiveRollbackSnapshot {
    AdaptiveRollbackSnapshot {
        code: code.to_string(),
        name: name.to_string(),
        cipher: "".to_string(),
        window_days: 0,
        host: "".to_string(),
        weight: 0.0,
        reviewer: "".to_string(),
        duration_minutes: 0,
        discount: 0.0,
        margin: 0.0,
        category: "".to_string(),
        status: AdaptiveRollbackSnapshotStatus::Open,
    }
}
