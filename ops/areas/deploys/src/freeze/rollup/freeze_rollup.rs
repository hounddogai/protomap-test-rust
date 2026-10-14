use super::*;

use acme_kit::text;

/// Where a freeze rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FreezeRollupStatus {
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

impl std::fmt::Display for FreezeRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze rollup. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeRollup {
    pub code: String,
    pub title: String,
    pub region: String,
    pub archived: bool,
    pub host: String,
    pub notes: String,
    pub amount: f64,
    pub status: FreezeRollupStatus,
}

impl FreezeRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeRollupStatus) -> bool {
        match self.status {
            FreezeRollupStatus::Idle => {
                match next {
                    FreezeRollupStatus::Warming | FreezeRollupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeRollupStatus::Warming => {
                return next == FreezeRollupStatus::Ready || next == FreezeRollupStatus::Stopped;
            }
            FreezeRollupStatus::Ready => {
                match next {
                    FreezeRollupStatus::Draining | FreezeRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeRollupStatus::Draining => {
                match next {
                    FreezeRollupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FreezeRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "freeze rollup", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> FreezeRollup {
        FreezeRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            region: self.region.clone(),
            archived: self.archived,
            host: self.host.clone(),
            notes: self.notes.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a freeze rollup in status idle.
pub fn new_freeze_rollup(code: &str, title: &str) -> FreezeRollup {
    FreezeRollup {
        code: code.to_string(),
        title: title.to_string(),
        region: "".to_string(),
        archived: false,
        host: "".to_string(),
        notes: "".to_string(),
        amount: 0.0,
        status: FreezeRollupStatus::Idle,
    }
}
