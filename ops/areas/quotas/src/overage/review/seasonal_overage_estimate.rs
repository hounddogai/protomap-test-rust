use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a seasonal overage estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalOverageEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalOverageEstimateStatus {
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

impl std::fmt::Display for SeasonalOverageEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal overage estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalOverageEstimate {
    pub code: String,
    pub title: String,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub name: String,
    pub scope: String,
    pub window_days: i64,
    pub status: SeasonalOverageEstimateStatus,
}

impl SeasonalOverageEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalOverageEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalOverageEstimateStatus) -> bool {
        match self.status {
            SeasonalOverageEstimateStatus::Idle => {
                match next {
                    SeasonalOverageEstimateStatus::Draining | SeasonalOverageEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalOverageEstimateStatus::Warming => {
                return next == SeasonalOverageEstimateStatus::Draining
                    || next == SeasonalOverageEstimateStatus::Stopped;
            }
            SeasonalOverageEstimateStatus::Ready => {
                match next {
                    SeasonalOverageEstimateStatus::Draining | SeasonalOverageEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalOverageEstimateStatus::Draining => {
                match next {
                    SeasonalOverageEstimateStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalOverageEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal overage estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_value(&self) -> f64 {
        (self.rate * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalOverageEstimate {
        SeasonalOverageEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            rate: self.rate,
            attributes: self.attributes.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a seasonal overage estimate in status idle.
pub fn new_seasonal_overage_estimate(code: &str, title: &str) -> SeasonalOverageEstimate {
    SeasonalOverageEstimate {
        code: code.to_string(),
        title: title.to_string(),
        rate: 0.0,
        attributes: BTreeMap::new(),
        name: "".to_string(),
        scope: "".to_string(),
        window_days: 0,
        status: SeasonalOverageEstimateStatus::Idle,
    }
}
