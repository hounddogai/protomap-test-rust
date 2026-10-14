use super::*;

use acme_kit::text;

/// Where a seasonal identity cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalIdentityCycleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl SeasonalIdentityCycleStatus {
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

impl std::fmt::Display for SeasonalIdentityCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal identity cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIdentityCycle {
    pub code: String,
    pub name: String,
    pub channel: String,
    pub source: String,
    pub updated_at: i64,
    pub rotation_days: i64,
    pub batch_size: i64,
    pub pinned: bool,
    pub attempts: i64,
    pub external_ref: String,
    pub status: SeasonalIdentityCycleStatus,
}

impl SeasonalIdentityCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalIdentityCycleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalIdentityCycleStatus) -> bool {
        match self.status {
            SeasonalIdentityCycleStatus::New => {
                match next {
                    SeasonalIdentityCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIdentityCycleStatus::Queued => {
                return next == SeasonalIdentityCycleStatus::Running || next == SeasonalIdentityCycleStatus::Done;
            }
            SeasonalIdentityCycleStatus::Running => {
                match next {
                    SeasonalIdentityCycleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIdentityCycleStatus::Done => {
                match next {
                    SeasonalIdentityCycleStatus::Failed => {
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

    pub fn move_to(&mut self, next: SeasonalIdentityCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal identity cycle {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.updated_at as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalIdentityCycle {
        SeasonalIdentityCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            channel: self.channel.clone(),
            source: self.source.clone(),
            updated_at: self.updated_at,
            rotation_days: self.rotation_days,
            batch_size: self.batch_size,
            pinned: self.pinned,
            attempts: self.attempts,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a seasonal identity cycle in status new.
pub fn new_seasonal_identity_cycle(code: &str, name: &str) -> SeasonalIdentityCycle {
    SeasonalIdentityCycle {
        code: code.to_string(),
        name: name.to_string(),
        channel: "".to_string(),
        source: "".to_string(),
        updated_at: 0,
        rotation_days: 0,
        batch_size: 0,
        pinned: false,
        attempts: 0,
        external_ref: "".to_string(),
        status: SeasonalIdentityCycleStatus::New,
    }
}
