use super::*;

use acme_kit::text;

/// Where a fallback region pin plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRegionPinPlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackRegionPinPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for FallbackRegionPinPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback region pin plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRegionPinPlan {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub cipher: String,
    pub owner_id: String,
    pub discount: f64,
    pub timezone: String,
    pub principal: String,
    pub notes: String,
    pub archived: bool,
    pub floor_value: f64,
    pub status: FallbackRegionPinPlanStatus,
}

impl FallbackRegionPinPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRegionPinPlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRegionPinPlanStatus) -> bool {
        match self.status {
            FallbackRegionPinPlanStatus::Planned => {
                match next {
                    FallbackRegionPinPlanStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRegionPinPlanStatus::Scheduled => {
                return next == FallbackRegionPinPlanStatus::InProgress;
            }
            FallbackRegionPinPlanStatus::InProgress => {
                match next {
                    FallbackRegionPinPlanStatus::Completed | FallbackRegionPinPlanStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRegionPinPlanStatus::Completed => {
                return next == FallbackRegionPinPlanStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRegionPinPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback region pin plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback region pin plan", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.floor_value - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRegionPinPlan {
        FallbackRegionPinPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            cipher: self.cipher.clone(),
            owner_id: self.owner_id.clone(),
            discount: self.discount,
            timezone: self.timezone.clone(),
            principal: self.principal.clone(),
            notes: self.notes.clone(),
            archived: self.archived,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback region pin plan in status planned.
pub fn new_fallback_region_pin_plan(code: &str, label: &str) -> FallbackRegionPinPlan {
    FallbackRegionPinPlan {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        cipher: "".to_string(),
        owner_id: "".to_string(),
        discount: 0.0,
        timezone: "".to_string(),
        principal: "".to_string(),
        notes: "".to_string(),
        archived: false,
        floor_value: 0.0,
        status: FallbackRegionPinPlanStatus::Planned,
    }
}
