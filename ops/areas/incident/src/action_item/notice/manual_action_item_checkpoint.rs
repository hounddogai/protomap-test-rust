use super::*;

use acme_kit::text;

/// Where a manual action item checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualActionItemCheckpointStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl ManualActionItemCheckpointStatus {
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

impl std::fmt::Display for ManualActionItemCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual action item checkpoint as the action item team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualActionItemCheckpoint {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub checksum: String,
    pub ends_at: i64,
    pub weight: f64,
    pub status: ManualActionItemCheckpointStatus,
}

impl ManualActionItemCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualActionItemCheckpointStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualActionItemCheckpointStatus) -> bool {
        match self.status {
            ManualActionItemCheckpointStatus::New => {
                return next == ManualActionItemCheckpointStatus::Done;
            }
            ManualActionItemCheckpointStatus::Queued => {
                return next == ManualActionItemCheckpointStatus::Done
                    || next == ManualActionItemCheckpointStatus::Failed;
            }
            ManualActionItemCheckpointStatus::Running => {
                match next {
                    ManualActionItemCheckpointStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualActionItemCheckpointStatus::Done => {
                match next {
                    ManualActionItemCheckpointStatus::Failed => {
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

    pub fn move_to(&mut self, next: ManualActionItemCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual action item checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual action item checkpoint", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ends_at as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualActionItemCheckpoint {
        ManualActionItemCheckpoint {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            checksum: self.checksum.clone(),
            ends_at: self.ends_at,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual action item checkpoint in status new.
pub fn new_manual_action_item_checkpoint(code: &str, title: &str) -> ManualActionItemCheckpoint {
    ManualActionItemCheckpoint {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        checksum: "".to_string(),
        ends_at: 0,
        weight: 0.0,
        status: ManualActionItemCheckpointStatus::New,
    }
}
