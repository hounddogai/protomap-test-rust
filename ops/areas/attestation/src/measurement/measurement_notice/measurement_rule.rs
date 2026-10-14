use super::*;

use acme_kit::text;

/// Where a measurement rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementRuleStatus {
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

impl std::fmt::Display for MeasurementRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementRule {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub discount: f64,
    pub maximum: f64,
    pub status: MeasurementRuleStatus,
}

impl MeasurementRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementRuleStatus) -> bool {
        match self.status {
            MeasurementRuleStatus::Idle => {
                return next == MeasurementRuleStatus::Ready;
            }
            MeasurementRuleStatus::Warming => {
                return next == MeasurementRuleStatus::Ready;
            }
            MeasurementRuleStatus::Ready => {
                return next == MeasurementRuleStatus::Stopped;
            }
            MeasurementRuleStatus::Draining => {
                match next {
                    MeasurementRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MeasurementRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        (self.discount * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementRule {
        MeasurementRule {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            discount: self.discount,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a measurement rule in status idle.
pub fn new_measurement_rule(code: &str, label: &str) -> MeasurementRule {
    MeasurementRule {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        discount: 0.0,
        maximum: 0.0,
        status: MeasurementRuleStatus::Idle,
    }
}
