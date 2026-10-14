use super::*;

use acme_kit::text;

/// Where a manual runbook checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRunbookCheckpointStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualRunbookCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ManualRunbookCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual runbook checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookCheckpoint {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub port: i64,
    pub score: f64,
    pub status: ManualRunbookCheckpointStatus,
}

impl ManualRunbookCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRunbookCheckpointStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRunbookCheckpointStatus) -> bool {
        match self.status {
            ManualRunbookCheckpointStatus::Idle => {
                return next == ManualRunbookCheckpointStatus::Draining;
            }
            ManualRunbookCheckpointStatus::Warming => {
                return next == ManualRunbookCheckpointStatus::Draining
                    || next == ManualRunbookCheckpointStatus::Stopped;
            }
            ManualRunbookCheckpointStatus::Ready => {
                match next {
                    ManualRunbookCheckpointStatus::Draining | ManualRunbookCheckpointStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRunbookCheckpointStatus::Draining => {
                match next {
                    ManualRunbookCheckpointStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ManualRunbookCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual runbook checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual runbook checkpoint", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ceiling - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRunbookCheckpoint {
        ManualRunbookCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            port: self.port,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual runbook checkpoint in status idle.
pub fn new_manual_runbook_checkpoint(code: &str, name: &str) -> ManualRunbookCheckpoint {
    ManualRunbookCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        port: 0,
        score: 0.0,
        status: ManualRunbookCheckpointStatus::Idle,
    }
}
