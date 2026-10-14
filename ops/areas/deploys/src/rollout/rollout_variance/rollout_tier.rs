use super::*;

use acme_kit::text;

/// Where a rollout tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RolloutTierStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RolloutTierStatus {
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

impl std::fmt::Display for RolloutTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rollout tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RolloutTier {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub quota_units: i64,
    pub segment: String,
    pub ratio: f64,
    pub contact_email: String,
    pub updated_at: i64,
    pub checksum: String,
    pub cluster: String,
    pub window_days: i64,
    pub status: RolloutTierStatus,
}

impl RolloutTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RolloutTierStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RolloutTierStatus) -> bool {
        match self.status {
            RolloutTierStatus::New => {
                match next {
                    RolloutTierStatus::Running | RolloutTierStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RolloutTierStatus::Queued => {
                return next == RolloutTierStatus::Failed;
            }
            RolloutTierStatus::Running => {
                return next == RolloutTierStatus::Done || next == RolloutTierStatus::Failed;
            }
            RolloutTierStatus::Done => {
                return next == RolloutTierStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RolloutTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollout tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.window_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RolloutTier {
        RolloutTier {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            quota_units: self.quota_units,
            segment: self.segment.clone(),
            ratio: self.ratio,
            contact_email: self.contact_email.clone(),
            updated_at: self.updated_at,
            checksum: self.checksum.clone(),
            cluster: self.cluster.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollout tier in status new.
pub fn new_rollout_tier(code: &str, title: &str) -> RolloutTier {
    RolloutTier {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        quota_units: 0,
        segment: "".to_string(),
        ratio: 0.0,
        contact_email: "".to_string(),
        updated_at: 0,
        checksum: "".to_string(),
        cluster: "".to_string(),
        window_days: 0,
        status: RolloutTierStatus::New,
    }
}
