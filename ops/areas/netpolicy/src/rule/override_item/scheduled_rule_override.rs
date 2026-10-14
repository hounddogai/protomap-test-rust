use super::*;

use acme_kit::text;

/// Where a scheduled rule override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledRuleOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledRuleOverrideStatus {
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

impl std::fmt::Display for ScheduledRuleOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled rule override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledRuleOverride {
    pub code: String,
    pub label: String,
    pub title: String,
    pub grace_days: i64,
    pub amount: f64,
    pub issuer: String,
    pub limit: i64,
    pub weight: f64,
    pub locale: String,
    pub source: String,
    pub duration_minutes: i64,
    pub status: ScheduledRuleOverrideStatus,
}

impl ScheduledRuleOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledRuleOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledRuleOverrideStatus) -> bool {
        match self.status {
            ScheduledRuleOverrideStatus::Idle => {
                return next == ScheduledRuleOverrideStatus::Ready;
            }
            ScheduledRuleOverrideStatus::Warming => {
                return next == ScheduledRuleOverrideStatus::Draining;
            }
            ScheduledRuleOverrideStatus::Ready => {
                return next == ScheduledRuleOverrideStatus::Draining || next == ScheduledRuleOverrideStatus::Stopped;
            }
            ScheduledRuleOverrideStatus::Draining => {
                match next {
                    ScheduledRuleOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ScheduledRuleOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled rule override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        (self.weight * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledRuleOverride {
        ScheduledRuleOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            amount: self.amount,
            issuer: self.issuer.clone(),
            limit: self.limit,
            weight: self.weight,
            locale: self.locale.clone(),
            source: self.source.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a scheduled rule override in status idle.
pub fn new_scheduled_rule_override(code: &str, label: &str) -> ScheduledRuleOverride {
    ScheduledRuleOverride {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        grace_days: 0,
        amount: 0.0,
        issuer: "".to_string(),
        limit: 0,
        weight: 0.0,
        locale: "".to_string(),
        source: "".to_string(),
        duration_minutes: 0,
        status: ScheduledRuleOverrideStatus::Idle,
    }
}
