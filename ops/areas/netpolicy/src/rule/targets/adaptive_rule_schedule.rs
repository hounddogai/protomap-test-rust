use super::*;

use acme_kit::text;

/// Where a adaptive rule schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRuleScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveRuleScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AdaptiveRuleScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive rule schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRuleSchedule {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub source: String,
    pub rotation_days: i64,
    pub budget: f64,
    pub grace_days: i64,
    pub ceiling: f64,
    pub quantity: i64,
    pub archived: bool,
    pub status: AdaptiveRuleScheduleStatus,
}

impl AdaptiveRuleSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRuleScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRuleScheduleStatus) -> bool {
        match self.status {
            AdaptiveRuleScheduleStatus::Draft => {
                return next == AdaptiveRuleScheduleStatus::Active || next == AdaptiveRuleScheduleStatus::Closed;
            }
            AdaptiveRuleScheduleStatus::Pending => {
                return next == AdaptiveRuleScheduleStatus::Active;
            }
            AdaptiveRuleScheduleStatus::Active => {
                return next == AdaptiveRuleScheduleStatus::Closed;
            }
            AdaptiveRuleScheduleStatus::Suspended => {
                return next == AdaptiveRuleScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRuleScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive rule schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        (self.ceiling * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRuleSchedule {
        AdaptiveRuleSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            source: self.source.clone(),
            rotation_days: self.rotation_days,
            budget: self.budget,
            grace_days: self.grace_days,
            ceiling: self.ceiling,
            quantity: self.quantity,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive rule schedule in status draft.
pub fn new_adaptive_rule_schedule(code: &str, title: &str) -> AdaptiveRuleSchedule {
    AdaptiveRuleSchedule {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        source: "".to_string(),
        rotation_days: 0,
        budget: 0.0,
        grace_days: 0,
        ceiling: 0.0,
        quantity: 0,
        archived: false,
        status: AdaptiveRuleScheduleStatus::Draft,
    }
}
