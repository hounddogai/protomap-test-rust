use super::*;

use acme_kit::text;

/// Where a legacy grant rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyGrantRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyGrantRollupStatus {
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

impl std::fmt::Display for LegacyGrantRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy grant rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyGrantRollup {
    pub code: String,
    pub title: String,
    pub maximum: f64,
    pub reviewer: String,
    pub audience: String,
    pub fingerprint: String,
    pub burst: i64,
    pub unit: String,
    pub key_id: String,
    pub status: LegacyGrantRollupStatus,
}

impl LegacyGrantRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyGrantRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyGrantRollupStatus) -> bool {
        match self.status {
            LegacyGrantRollupStatus::Idle => {
                return next == LegacyGrantRollupStatus::Stopped;
            }
            LegacyGrantRollupStatus::Warming => {
                match next {
                    LegacyGrantRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyGrantRollupStatus::Ready => {
                match next {
                    LegacyGrantRollupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyGrantRollupStatus::Draining => {
                return next == LegacyGrantRollupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyGrantRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy grant rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.burst as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyGrantRollup {
        LegacyGrantRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            maximum: self.maximum,
            reviewer: self.reviewer.clone(),
            audience: self.audience.clone(),
            fingerprint: self.fingerprint.clone(),
            burst: self.burst,
            unit: self.unit.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a legacy grant rollup in status idle.
pub fn new_legacy_grant_rollup(code: &str, title: &str) -> LegacyGrantRollup {
    LegacyGrantRollup {
        code: code.to_string(),
        title: title.to_string(),
        maximum: 0.0,
        reviewer: "".to_string(),
        audience: "".to_string(),
        fingerprint: "".to_string(),
        burst: 0,
        unit: "".to_string(),
        key_id: "".to_string(),
        status: LegacyGrantRollupStatus::Idle,
    }
}
