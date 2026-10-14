use super::*;

use acme_kit::text;

/// Where a nonce checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceCheckpointStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl NonceCheckpointStatus {
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

impl std::fmt::Display for NonceCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a nonce checkpoint and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceCheckpoint {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub priority: i64,
    pub issuer: String,
    pub status: NonceCheckpointStatus,
}

impl NonceCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceCheckpointStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceCheckpointStatus) -> bool {
        match self.status {
            NonceCheckpointStatus::Idle => {
                return next == NonceCheckpointStatus::Warming || next == NonceCheckpointStatus::Ready;
            }
            NonceCheckpointStatus::Warming => {
                match next {
                    NonceCheckpointStatus::Draining | NonceCheckpointStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceCheckpointStatus::Ready => {
                match next {
                    NonceCheckpointStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceCheckpointStatus::Draining => {
                match next {
                    NonceCheckpointStatus::Stopped => {
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

    pub fn move_to(&mut self, next: NonceCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceCheckpoint {
        NonceCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            priority: self.priority,
            issuer: self.issuer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a nonce checkpoint in status idle.
pub fn new_nonce_checkpoint(code: &str, name: &str) -> NonceCheckpoint {
    NonceCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        priority: 0,
        issuer: "".to_string(),
        status: NonceCheckpointStatus::Idle,
    }
}
