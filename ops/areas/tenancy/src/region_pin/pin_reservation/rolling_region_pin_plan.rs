use super::*;

use acme_kit::text;

/// Where a rolling region pin plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRegionPinPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingRegionPinPlanStatus {
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

impl std::fmt::Display for RollingRegionPinPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling region pin plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinPlan {
    pub code: String,
    pub title: String,
    pub quota_units: i64,
    pub currency: String,
    pub source: String,
    pub threshold: f64,
    pub status: RollingRegionPinPlanStatus,
}

impl RollingRegionPinPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRegionPinPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRegionPinPlanStatus) -> bool {
        match self.status {
            RollingRegionPinPlanStatus::Idle => {
                match next {
                    RollingRegionPinPlanStatus::Draining | RollingRegionPinPlanStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRegionPinPlanStatus::Warming => {
                match next {
                    RollingRegionPinPlanStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRegionPinPlanStatus::Ready => {
                match next {
                    RollingRegionPinPlanStatus::Draining | RollingRegionPinPlanStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRegionPinPlanStatus::Draining => {
                match next {
                    RollingRegionPinPlanStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollingRegionPinPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling region pin plan {} cannot move from {} to {}",
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
        ((self.threshold - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRegionPinPlan {
        RollingRegionPinPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            source: self.source.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling region pin plan in status idle.
pub fn new_rolling_region_pin_plan(code: &str, title: &str) -> RollingRegionPinPlan {
    RollingRegionPinPlan {
        code: code.to_string(),
        title: title.to_string(),
        quota_units: 0,
        currency: "".to_string(),
        source: "".to_string(),
        threshold: 0.0,
        status: RollingRegionPinPlanStatus::Idle,
    }
}
