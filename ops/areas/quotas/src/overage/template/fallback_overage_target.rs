use super::*;

use acme_kit::text;

/// Where a fallback overage target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackOverageTargetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FallbackOverageTargetStatus {
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

impl std::fmt::Display for FallbackOverageTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback overage target as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackOverageTarget {
    pub code: String,
    pub name: String,
    pub checksum: String,
    pub ceiling: f64,
    pub region: String,
    pub category: String,
    pub grace_days: i64,
    pub source: String,
    pub status: FallbackOverageTargetStatus,
}

impl FallbackOverageTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackOverageTargetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackOverageTargetStatus) -> bool {
        match self.status {
            FallbackOverageTargetStatus::Idle => {
                return next == FallbackOverageTargetStatus::Warming;
            }
            FallbackOverageTargetStatus::Warming => {
                match next {
                    FallbackOverageTargetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageTargetStatus::Ready => {
                match next {
                    FallbackOverageTargetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackOverageTargetStatus::Draining => {
                match next {
                    FallbackOverageTargetStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FallbackOverageTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback overage target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ceiling + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackOverageTarget {
        FallbackOverageTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            checksum: self.checksum.clone(),
            ceiling: self.ceiling,
            region: self.region.clone(),
            category: self.category.clone(),
            grace_days: self.grace_days,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a fallback overage target in status idle.
pub fn new_fallback_overage_target(code: &str, name: &str) -> FallbackOverageTarget {
    FallbackOverageTarget {
        code: code.to_string(),
        name: name.to_string(),
        checksum: "".to_string(),
        ceiling: 0.0,
        region: "".to_string(),
        category: "".to_string(),
        grace_days: 0,
        source: "".to_string(),
        status: FallbackOverageTargetStatus::Idle,
    }
}
