use super::*;

use acme_kit::text;

/// Where a adaptive release tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveReleaseTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveReleaseTierStatus {
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

impl std::fmt::Display for AdaptiveReleaseTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive release tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveReleaseTier {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub starts_at: i64,
    pub duration_minutes: i64,
    pub namespace: String,
    pub score: f64,
    pub status: AdaptiveReleaseTierStatus,
}

impl AdaptiveReleaseTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveReleaseTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveReleaseTierStatus) -> bool {
        match self.status {
            AdaptiveReleaseTierStatus::Idle => {
                match next {
                    AdaptiveReleaseTierStatus::Ready | AdaptiveReleaseTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveReleaseTierStatus::Warming => {
                return next == AdaptiveReleaseTierStatus::Ready || next == AdaptiveReleaseTierStatus::Draining;
            }
            AdaptiveReleaseTierStatus::Ready => {
                match next {
                    AdaptiveReleaseTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveReleaseTierStatus::Draining => {
                match next {
                    AdaptiveReleaseTierStatus::Stopped => {
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

    pub fn move_to(&mut self, next: AdaptiveReleaseTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive release tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveReleaseTier {
        AdaptiveReleaseTier {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            starts_at: self.starts_at,
            duration_minutes: self.duration_minutes,
            namespace: self.namespace.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a adaptive release tier in status idle.
pub fn new_adaptive_release_tier(code: &str, label: &str) -> AdaptiveReleaseTier {
    AdaptiveReleaseTier {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        starts_at: 0,
        duration_minutes: 0,
        namespace: "".to_string(),
        score: 0.0,
        status: AdaptiveReleaseTierStatus::Idle,
    }
}
