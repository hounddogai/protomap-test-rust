use super::*;

use acme_kit::text;

/// Where a rollback snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackSnapshotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollbackSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for RollbackSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollback snapshot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackSnapshot {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub audience: String,
    pub minimum: f64,
    pub source: String,
    pub expires_at: i64,
    pub segment: String,
    pub namespace: String,
    pub status: RollbackSnapshotStatus,
}

impl RollbackSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackSnapshotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackSnapshotStatus) -> bool {
        match self.status {
            RollbackSnapshotStatus::New => {
                match next {
                    RollbackSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSnapshotStatus::Queued => {
                match next {
                    RollbackSnapshotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSnapshotStatus::Running => {
                match next {
                    RollbackSnapshotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackSnapshotStatus::Done => {
                return next == RollbackSnapshotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.expires_at as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackSnapshot {
        RollbackSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            minimum: self.minimum,
            source: self.source.clone(),
            expires_at: self.expires_at,
            segment: self.segment.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rollback snapshot in status new.
pub fn new_rollback_snapshot(code: &str, label: &str) -> RollbackSnapshot {
    RollbackSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        audience: "".to_string(),
        minimum: 0.0,
        source: "".to_string(),
        expires_at: 0,
        segment: "".to_string(),
        namespace: "".to_string(),
        status: RollbackSnapshotStatus::New,
    }
}
