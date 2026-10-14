use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a measurement template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementTemplateStatus {
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

impl std::fmt::Display for MeasurementTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a measurement template and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementTemplate {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub grace_days: i64,
    pub owner_id: String,
    pub status: MeasurementTemplateStatus,
}

impl MeasurementTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementTemplateStatus) -> bool {
        match self.status {
            MeasurementTemplateStatus::Idle => {
                match next {
                    MeasurementTemplateStatus::Warming | MeasurementTemplateStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementTemplateStatus::Warming => {
                return next == MeasurementTemplateStatus::Ready || next == MeasurementTemplateStatus::Draining;
            }
            MeasurementTemplateStatus::Ready => {
                match next {
                    MeasurementTemplateStatus::Draining | MeasurementTemplateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementTemplateStatus::Draining => {
                return next == MeasurementTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> MeasurementTemplate {
        MeasurementTemplate {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a measurement template in status idle.
pub fn new_measurement_template(code: &str, name: &str) -> MeasurementTemplate {
    MeasurementTemplate {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        grace_days: 0,
        owner_id: "".to_string(),
        status: MeasurementTemplateStatus::Idle,
    }
}
