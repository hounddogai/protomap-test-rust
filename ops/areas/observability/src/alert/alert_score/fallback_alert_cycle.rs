use super::*;

use acme_kit::text;

/// Where a fallback alert cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackAlertCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackAlertCycleStatus {
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

impl std::fmt::Display for FallbackAlertCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback alert cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackAlertCycle {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub maximum: f64,
    pub channel: String,
    pub tenant_id: String,
    pub archived: bool,
    pub status: FallbackAlertCycleStatus,
}

impl FallbackAlertCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackAlertCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackAlertCycleStatus) -> bool {
        match self.status {
            FallbackAlertCycleStatus::New => {
                return next == FallbackAlertCycleStatus::Queued;
            }
            FallbackAlertCycleStatus::Queued => {
                match next {
                    FallbackAlertCycleStatus::Running | FallbackAlertCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackAlertCycleStatus::Running => {
                return next == FallbackAlertCycleStatus::Done;
            }
            FallbackAlertCycleStatus::Done => {
                return next == FallbackAlertCycleStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackAlertCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback alert cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.maximum + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackAlertCycle {
        FallbackAlertCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            maximum: self.maximum,
            channel: self.channel.clone(),
            tenant_id: self.tenant_id.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback alert cycle in status new.
pub fn new_fallback_alert_cycle(code: &str, name: &str) -> FallbackAlertCycle {
    FallbackAlertCycle {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        maximum: 0.0,
        channel: "".to_string(),
        tenant_id: "".to_string(),
        archived: false,
        status: FallbackAlertCycleStatus::New,
    }
}
