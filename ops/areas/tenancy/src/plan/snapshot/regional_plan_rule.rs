use super::*;

use acme_kit::text;

/// Where a regional plan rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalPlanRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RegionalPlanRuleStatus {
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

impl std::fmt::Display for RegionalPlanRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional plan rule as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalPlanRule {
    pub code: String,
    pub title: String,
    pub duration_minutes: i64,
    pub margin: f64,
    pub threshold: f64,
    pub updated_at: i64,
    pub grace_days: i64,
    pub status: RegionalPlanRuleStatus,
}

impl RegionalPlanRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalPlanRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalPlanRuleStatus) -> bool {
        match self.status {
            RegionalPlanRuleStatus::Planned => {
                return next == RegionalPlanRuleStatus::Completed || next == RegionalPlanRuleStatus::Cancelled;
            }
            RegionalPlanRuleStatus::Scheduled => {
                return next == RegionalPlanRuleStatus::InProgress || next == RegionalPlanRuleStatus::Completed;
            }
            RegionalPlanRuleStatus::InProgress => {
                return next == RegionalPlanRuleStatus::Completed || next == RegionalPlanRuleStatus::Cancelled;
            }
            RegionalPlanRuleStatus::Completed => {
                match next {
                    RegionalPlanRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: RegionalPlanRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional plan rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.margin * (self.updated_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalPlanRule {
        RegionalPlanRule {
            code: self.code.clone(),
            title: self.title.clone(),
            duration_minutes: self.duration_minutes,
            margin: self.margin,
            threshold: self.threshold,
            updated_at: self.updated_at,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a regional plan rule in status planned.
pub fn new_regional_plan_rule(code: &str, title: &str) -> RegionalPlanRule {
    RegionalPlanRule {
        code: code.to_string(),
        title: title.to_string(),
        duration_minutes: 0,
        margin: 0.0,
        threshold: 0.0,
        updated_at: 0,
        grace_days: 0,
        status: RegionalPlanRuleStatus::Planned,
    }
}
