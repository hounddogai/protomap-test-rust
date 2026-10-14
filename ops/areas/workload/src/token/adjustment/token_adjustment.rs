use super::*;

use acme_kit::text;

/// Where a token adjustment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenAdjustmentStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenAdjustmentStatus {
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

impl std::fmt::Display for TokenAdjustmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token adjustment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenAdjustment {
    pub code: String,
    pub name: String,
    pub margin: f64,
    pub maximum: f64,
    pub active: bool,
    pub status: TokenAdjustmentStatus,
}

impl TokenAdjustment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenAdjustmentStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenAdjustmentStatus) -> bool {
        match self.status {
            TokenAdjustmentStatus::Idle => {
                return next == TokenAdjustmentStatus::Stopped;
            }
            TokenAdjustmentStatus::Warming => {
                match next {
                    TokenAdjustmentStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenAdjustmentStatus::Ready => {
                match next {
                    TokenAdjustmentStatus::Draining | TokenAdjustmentStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenAdjustmentStatus::Draining => {
                return next == TokenAdjustmentStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenAdjustmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token adjustment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.maximum - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenAdjustment {
        TokenAdjustment {
            code: self.code.clone(),
            name: self.name.clone(),
            margin: self.margin,
            maximum: self.maximum,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token adjustment in status idle.
pub fn new_token_adjustment(code: &str, name: &str) -> TokenAdjustment {
    TokenAdjustment {
        code: code.to_string(),
        name: name.to_string(),
        margin: 0.0,
        maximum: 0.0,
        active: false,
        status: TokenAdjustmentStatus::Idle,
    }
}
