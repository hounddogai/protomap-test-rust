use super::*;

use acme_kit::text;

/// Where a legacy limit plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyLimitPlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LegacyLimitPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LegacyLimitPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy limit plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyLimitPlan {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub window_days: i64,
    pub weight: f64,
    pub retries: i64,
    pub status: LegacyLimitPlanStatus,
}

impl LegacyLimitPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyLimitPlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyLimitPlanStatus) -> bool {
        match self.status {
            LegacyLimitPlanStatus::Proposed => {
                return next == LegacyLimitPlanStatus::Withdrawn;
            }
            LegacyLimitPlanStatus::Approved => {
                match next {
                    LegacyLimitPlanStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyLimitPlanStatus::Rejected => {
                return next == LegacyLimitPlanStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyLimitPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy limit plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy limit plan", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.window_days as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyLimitPlan {
        LegacyLimitPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            window_days: self.window_days,
            weight: self.weight,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy limit plan in status proposed.
pub fn new_legacy_limit_plan(code: &str, name: &str) -> LegacyLimitPlan {
    LegacyLimitPlan {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        window_days: 0,
        weight: 0.0,
        retries: 0,
        status: LegacyLimitPlanStatus::Proposed,
    }
}
