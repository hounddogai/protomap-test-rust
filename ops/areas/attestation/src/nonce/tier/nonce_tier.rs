use super::*;

use acme_kit::text;

/// Where a nonce tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NonceTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl NonceTierStatus {
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

impl std::fmt::Display for NonceTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One nonce tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct NonceTier {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub weight: f64,
    pub visible: bool,
    pub status: NonceTierStatus,
}

impl NonceTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            NonceTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: NonceTierStatus) -> bool {
        match self.status {
            NonceTierStatus::Idle => {
                match next {
                    NonceTierStatus::Ready | NonceTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceTierStatus::Warming => {
                return next == NonceTierStatus::Draining;
            }
            NonceTierStatus::Ready => {
                match next {
                    NonceTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            NonceTierStatus::Draining => {
                return next == NonceTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: NonceTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "nonce tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.weight - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> NonceTier {
        NonceTier {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            weight: self.weight,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a nonce tier in status idle.
pub fn new_nonce_tier(code: &str, label: &str) -> NonceTier {
    NonceTier {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        weight: 0.0,
        visible: false,
        status: NonceTierStatus::Idle,
    }
}
