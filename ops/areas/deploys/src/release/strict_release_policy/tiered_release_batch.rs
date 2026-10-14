use super::*;

use acme_kit::text;

/// Where a tiered release batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredReleaseBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TieredReleaseBatchStatus {
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

impl std::fmt::Display for TieredReleaseBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered release batch as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredReleaseBatch {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub reviewer: String,
    pub version: i64,
    pub status: TieredReleaseBatchStatus,
}

impl TieredReleaseBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredReleaseBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredReleaseBatchStatus) -> bool {
        match self.status {
            TieredReleaseBatchStatus::New => {
                match next {
                    TieredReleaseBatchStatus::Queued | TieredReleaseBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredReleaseBatchStatus::Queued => {
                return next == TieredReleaseBatchStatus::Done;
            }
            TieredReleaseBatchStatus::Running => {
                match next {
                    TieredReleaseBatchStatus::Done | TieredReleaseBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredReleaseBatchStatus::Done => {
                return next == TieredReleaseBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredReleaseBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered release batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.version as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredReleaseBatch {
        TieredReleaseBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            reviewer: self.reviewer.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered release batch in status new.
pub fn new_tiered_release_batch(code: &str, name: &str) -> TieredReleaseBatch {
    TieredReleaseBatch {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        reviewer: "".to_string(),
        version: 0,
        status: TieredReleaseBatchStatus::New,
    }
}
