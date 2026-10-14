use super::*;

use acme_kit::text;

/// Where a pending nonce estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingNonceEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingNonceEstimateStatus {
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

impl std::fmt::Display for PendingNonceEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending nonce estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingNonceEstimate {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub label: String,
    pub margin: f64,
    pub weight: f64,
    pub visible: bool,
    pub audience: String,
    pub amount: f64,
    pub pinned: bool,
    pub status: PendingNonceEstimateStatus,
}

impl PendingNonceEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingNonceEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingNonceEstimateStatus) -> bool {
        match self.status {
            PendingNonceEstimateStatus::Idle => {
                return next == PendingNonceEstimateStatus::Draining || next == PendingNonceEstimateStatus::Stopped;
            }
            PendingNonceEstimateStatus::Warming => {
                return next == PendingNonceEstimateStatus::Stopped;
            }
            PendingNonceEstimateStatus::Ready => {
                match next {
                    PendingNonceEstimateStatus::Draining | PendingNonceEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingNonceEstimateStatus::Draining => {
                match next {
                    PendingNonceEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingNonceEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending nonce estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_total(&self) -> f64 {
        ((self.window_days as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingNonceEstimate {
        PendingNonceEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            label: self.label.clone(),
            margin: self.margin,
            weight: self.weight,
            visible: self.visible,
            audience: self.audience.clone(),
            amount: self.amount,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending nonce estimate in status idle.
pub fn new_pending_nonce_estimate(code: &str, name: &str) -> PendingNonceEstimate {
    PendingNonceEstimate {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        label: "".to_string(),
        margin: 0.0,
        weight: 0.0,
        visible: false,
        audience: "".to_string(),
        amount: 0.0,
        pinned: false,
        status: PendingNonceEstimateStatus::Idle,
    }
}
