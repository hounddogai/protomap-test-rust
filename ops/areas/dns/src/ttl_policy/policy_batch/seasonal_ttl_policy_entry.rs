use super::*;

use acme_kit::text;

/// Where a seasonal ttl policy entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalTtlPolicyEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalTtlPolicyEntryStatus {
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

impl std::fmt::Display for SeasonalTtlPolicyEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal ttl policy entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalTtlPolicyEntry {
    pub code: String,
    pub title: String,
    pub channel: String,
    pub retries: i64,
    pub batch_size: i64,
    pub score: f64,
    pub cost: f64,
    pub port: i64,
    pub status: SeasonalTtlPolicyEntryStatus,
}

impl SeasonalTtlPolicyEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalTtlPolicyEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalTtlPolicyEntryStatus) -> bool {
        match self.status {
            SeasonalTtlPolicyEntryStatus::Idle => {
                return next == SeasonalTtlPolicyEntryStatus::Ready || next == SeasonalTtlPolicyEntryStatus::Stopped;
            }
            SeasonalTtlPolicyEntryStatus::Warming => {
                match next {
                    SeasonalTtlPolicyEntryStatus::Ready | SeasonalTtlPolicyEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTtlPolicyEntryStatus::Ready => {
                match next {
                    SeasonalTtlPolicyEntryStatus::Draining | SeasonalTtlPolicyEntryStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalTtlPolicyEntryStatus::Draining => {
                match next {
                    SeasonalTtlPolicyEntryStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalTtlPolicyEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal ttl policy entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.retries as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalTtlPolicyEntry {
        SeasonalTtlPolicyEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            channel: self.channel.clone(),
            retries: self.retries,
            batch_size: self.batch_size,
            score: self.score,
            cost: self.cost,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a seasonal ttl policy entry in status idle.
pub fn new_seasonal_ttl_policy_entry(code: &str, title: &str) -> SeasonalTtlPolicyEntry {
    SeasonalTtlPolicyEntry {
        code: code.to_string(),
        title: title.to_string(),
        channel: "".to_string(),
        retries: 0,
        batch_size: 0,
        score: 0.0,
        cost: 0.0,
        port: 0,
        status: SeasonalTtlPolicyEntryStatus::Idle,
    }
}
