use super::*;

use acme_kit::text;

/// Where a adaptive sink tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveSinkTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl AdaptiveSinkTierStatus {
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

impl std::fmt::Display for AdaptiveSinkTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive sink tier, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveSinkTier {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub visible: bool,
    pub batch_size: i64,
    pub host: String,
    pub limit: i64,
    pub description: String,
    pub rotation_days: i64,
    pub fingerprint: String,
    pub status: AdaptiveSinkTierStatus,
}

impl AdaptiveSinkTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveSinkTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveSinkTierStatus) -> bool {
        match self.status {
            AdaptiveSinkTierStatus::New => {
                return next == AdaptiveSinkTierStatus::Running || next == AdaptiveSinkTierStatus::Failed;
            }
            AdaptiveSinkTierStatus::Queued => {
                match next {
                    AdaptiveSinkTierStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveSinkTierStatus::Running => {
                return next == AdaptiveSinkTierStatus::Done;
            }
            AdaptiveSinkTierStatus::Done => {
                match next {
                    AdaptiveSinkTierStatus::Failed => {
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

    pub fn move_to(&mut self, next: AdaptiveSinkTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive sink tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive sink tier", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.rotation_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveSinkTier {
        AdaptiveSinkTier {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            visible: self.visible,
            batch_size: self.batch_size,
            host: self.host.clone(),
            limit: self.limit,
            description: self.description.clone(),
            rotation_days: self.rotation_days,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive sink tier in status new.
pub fn new_adaptive_sink_tier(code: &str, title: &str) -> AdaptiveSinkTier {
    AdaptiveSinkTier {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        visible: false,
        batch_size: 0,
        host: "".to_string(),
        limit: 0,
        description: "".to_string(),
        rotation_days: 0,
        fingerprint: "".to_string(),
        status: AdaptiveSinkTierStatus::New,
    }
}
