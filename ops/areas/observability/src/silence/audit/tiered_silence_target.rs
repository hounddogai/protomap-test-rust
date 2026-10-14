use super::*;

use acme_kit::text;

/// Where a tiered silence target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSilenceTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredSilenceTargetStatus {
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

impl std::fmt::Display for TieredSilenceTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered silence target and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSilenceTarget {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub description: String,
    pub active: bool,
    pub approved: bool,
    pub unit: String,
    pub title: String,
    pub namespace: String,
    pub status: TieredSilenceTargetStatus,
}

impl TieredSilenceTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSilenceTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSilenceTargetStatus) -> bool {
        match self.status {
            TieredSilenceTargetStatus::Idle => {
                return next == TieredSilenceTargetStatus::Warming || next == TieredSilenceTargetStatus::Ready;
            }
            TieredSilenceTargetStatus::Warming => {
                return next == TieredSilenceTargetStatus::Draining;
            }
            TieredSilenceTargetStatus::Ready => {
                return next == TieredSilenceTargetStatus::Draining || next == TieredSilenceTargetStatus::Stopped;
            }
            TieredSilenceTargetStatus::Draining => {
                match next {
                    TieredSilenceTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TieredSilenceTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered silence target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TieredSilenceTarget {
        TieredSilenceTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            description: self.description.clone(),
            active: self.active,
            approved: self.approved,
            unit: self.unit.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a tiered silence target in status idle.
pub fn new_tiered_silence_target(code: &str, name: &str) -> TieredSilenceTarget {
    TieredSilenceTarget {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        description: "".to_string(),
        active: false,
        approved: false,
        unit: "".to_string(),
        title: "".to_string(),
        namespace: "".to_string(),
        status: TieredSilenceTargetStatus::Idle,
    }
}
