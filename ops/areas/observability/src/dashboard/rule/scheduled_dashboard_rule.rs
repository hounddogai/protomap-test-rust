use super::*;

use acme_kit::text;

/// Where a scheduled dashboard rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDashboardRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledDashboardRuleStatus {
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

impl std::fmt::Display for ScheduledDashboardRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled dashboard rule as the dashboard team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDashboardRule {
    pub code: String,
    pub label: String,
    pub contact_email: String,
    pub capacity: i64,
    pub grace_days: i64,
    pub batch_size: i64,
    pub minimum: f64,
    pub principal: String,
    pub category: String,
    pub port: i64,
    pub maximum: f64,
    pub status: ScheduledDashboardRuleStatus,
}

impl ScheduledDashboardRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDashboardRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDashboardRuleStatus) -> bool {
        match self.status {
            ScheduledDashboardRuleStatus::Idle => {
                return next == ScheduledDashboardRuleStatus::Warming || next == ScheduledDashboardRuleStatus::Stopped;
            }
            ScheduledDashboardRuleStatus::Warming => {
                return next == ScheduledDashboardRuleStatus::Stopped;
            }
            ScheduledDashboardRuleStatus::Ready => {
                return next == ScheduledDashboardRuleStatus::Draining || next == ScheduledDashboardRuleStatus::Stopped;
            }
            ScheduledDashboardRuleStatus::Draining => {
                return next == ScheduledDashboardRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledDashboardRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled dashboard rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled dashboard rule", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.grace_days as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDashboardRule {
        ScheduledDashboardRule {
            code: self.code.clone(),
            label: self.label.clone(),
            contact_email: self.contact_email.clone(),
            capacity: self.capacity,
            grace_days: self.grace_days,
            batch_size: self.batch_size,
            minimum: self.minimum,
            principal: self.principal.clone(),
            category: self.category.clone(),
            port: self.port,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a scheduled dashboard rule in status idle.
pub fn new_scheduled_dashboard_rule(code: &str, label: &str) -> ScheduledDashboardRule {
    ScheduledDashboardRule {
        code: code.to_string(),
        label: label.to_string(),
        contact_email: "".to_string(),
        capacity: 0,
        grace_days: 0,
        batch_size: 0,
        minimum: 0.0,
        principal: "".to_string(),
        category: "".to_string(),
        port: 0,
        maximum: 0.0,
        status: ScheduledDashboardRuleStatus::Idle,
    }
}
