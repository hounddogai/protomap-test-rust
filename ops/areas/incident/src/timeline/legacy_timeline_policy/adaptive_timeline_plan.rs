use super::*;

use acme_kit::text;

/// Where a adaptive timeline plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveTimelinePlanStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveTimelinePlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveTimelinePlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive timeline plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveTimelinePlan {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub attempts: i64,
    pub reviewer: String,
    pub status: AdaptiveTimelinePlanStatus,
}

impl AdaptiveTimelinePlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveTimelinePlanStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveTimelinePlanStatus) -> bool {
        match self.status {
            AdaptiveTimelinePlanStatus::Proposed => {
                return next == AdaptiveTimelinePlanStatus::Rejected || next == AdaptiveTimelinePlanStatus::Withdrawn;
            }
            AdaptiveTimelinePlanStatus::Approved => {
                return next == AdaptiveTimelinePlanStatus::Withdrawn;
            }
            AdaptiveTimelinePlanStatus::Rejected => {
                match next {
                    AdaptiveTimelinePlanStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: AdaptiveTimelinePlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive timeline plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "adaptive timeline plan", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveTimelinePlan {
        AdaptiveTimelinePlan {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            attempts: self.attempts,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive timeline plan in status proposed.
pub fn new_adaptive_timeline_plan(code: &str, label: &str) -> AdaptiveTimelinePlan {
    AdaptiveTimelinePlan {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        attempts: 0,
        reviewer: "".to_string(),
        status: AdaptiveTimelinePlanStatus::Proposed,
    }
}
