use super::*;

use acme_kit::text;

/// Where a adaptive plan schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptivePlanScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptivePlanScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptivePlanScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive plan schedule as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptivePlanSchedule {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub fingerprint: String,
    pub host: String,
    pub owner_id: String,
    pub audience: String,
    pub source: String,
    pub quantity: i64,
    pub status: AdaptivePlanScheduleStatus,
}

impl AdaptivePlanSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptivePlanScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptivePlanScheduleStatus) -> bool {
        match self.status {
            AdaptivePlanScheduleStatus::Created => {
                return next == AdaptivePlanScheduleStatus::Validated;
            }
            AdaptivePlanScheduleStatus::Validated => {
                return next == AdaptivePlanScheduleStatus::Published || next == AdaptivePlanScheduleStatus::Retired;
            }
            AdaptivePlanScheduleStatus::Published => {
                match next {
                    AdaptivePlanScheduleStatus::Retired => {
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

    pub fn move_to(&mut self, next: AdaptivePlanScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive plan schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> AdaptivePlanSchedule {
        AdaptivePlanSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            fingerprint: self.fingerprint.clone(),
            host: self.host.clone(),
            owner_id: self.owner_id.clone(),
            audience: self.audience.clone(),
            source: self.source.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive plan schedule in status created.
pub fn new_adaptive_plan_schedule(code: &str, label: &str) -> AdaptivePlanSchedule {
    AdaptivePlanSchedule {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        fingerprint: "".to_string(),
        host: "".to_string(),
        owner_id: "".to_string(),
        audience: "".to_string(),
        source: "".to_string(),
        quantity: 0,
        status: AdaptivePlanScheduleStatus::Created,
    }
}
