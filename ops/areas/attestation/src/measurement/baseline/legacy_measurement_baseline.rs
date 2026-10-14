use super::*;

use acme_kit::text;

/// Where a legacy measurement baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyMeasurementBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyMeasurementBaselineStatus {
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

impl std::fmt::Display for LegacyMeasurementBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy measurement baseline as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyMeasurementBaseline {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub principal: String,
    pub visible: bool,
    pub timezone: String,
    pub threshold: f64,
    pub name: String,
    pub status: LegacyMeasurementBaselineStatus,
}

impl LegacyMeasurementBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyMeasurementBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyMeasurementBaselineStatus) -> bool {
        match self.status {
            LegacyMeasurementBaselineStatus::Idle => {
                return next == LegacyMeasurementBaselineStatus::Ready;
            }
            LegacyMeasurementBaselineStatus::Warming => {
                match next {
                    LegacyMeasurementBaselineStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementBaselineStatus::Ready => {
                match next {
                    LegacyMeasurementBaselineStatus::Draining | LegacyMeasurementBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyMeasurementBaselineStatus::Draining => {
                match next {
                    LegacyMeasurementBaselineStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LegacyMeasurementBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy measurement baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.threshold * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyMeasurementBaseline {
        LegacyMeasurementBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            principal: self.principal.clone(),
            visible: self.visible,
            timezone: self.timezone.clone(),
            threshold: self.threshold,
            name: self.name.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a legacy measurement baseline in status idle.
pub fn new_legacy_measurement_baseline(code: &str, label: &str) -> LegacyMeasurementBaseline {
    LegacyMeasurementBaseline {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        principal: "".to_string(),
        visible: false,
        timezone: "".to_string(),
        threshold: 0.0,
        name: "".to_string(),
        status: LegacyMeasurementBaselineStatus::Idle,
    }
}
