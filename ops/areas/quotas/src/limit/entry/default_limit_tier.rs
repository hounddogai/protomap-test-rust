use super::*;

use acme_kit::text;

/// Where a default limit tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultLimitTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultLimitTierStatus {
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

impl std::fmt::Display for DefaultLimitTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default limit tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultLimitTier {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub ttl_seconds: i64,
    pub retries: i64,
    pub key_id: String,
    pub minimum: f64,
    pub status: DefaultLimitTierStatus,
}

impl DefaultLimitTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultLimitTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultLimitTierStatus) -> bool {
        match self.status {
            DefaultLimitTierStatus::Idle => {
                return next == DefaultLimitTierStatus::Ready || next == DefaultLimitTierStatus::Stopped;
            }
            DefaultLimitTierStatus::Warming => {
                return next == DefaultLimitTierStatus::Ready || next == DefaultLimitTierStatus::Stopped;
            }
            DefaultLimitTierStatus::Ready => {
                return next == DefaultLimitTierStatus::Stopped;
            }
            DefaultLimitTierStatus::Draining => {
                return next == DefaultLimitTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultLimitTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default limit tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultLimitTier {
        DefaultLimitTier {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            ttl_seconds: self.ttl_seconds,
            retries: self.retries,
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a default limit tier in status idle.
pub fn new_default_limit_tier(code: &str, title: &str) -> DefaultLimitTier {
    DefaultLimitTier {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        ttl_seconds: 0,
        retries: 0,
        key_id: "".to_string(),
        minimum: 0.0,
        status: DefaultLimitTierStatus::Idle,
    }
}
