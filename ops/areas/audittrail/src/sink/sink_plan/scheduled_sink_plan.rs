use super::*;

use acme_kit::text;

/// Where a scheduled sink plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledSinkPlanStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledSinkPlanStatus {
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

impl std::fmt::Display for ScheduledSinkPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled sink plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledSinkPlan {
    pub code: String,
    pub title: String,
    pub external_ref: String,
    pub amount: f64,
    pub principal: String,
    pub ttl_seconds: i64,
    pub status: ScheduledSinkPlanStatus,
}

impl ScheduledSinkPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledSinkPlanStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledSinkPlanStatus) -> bool {
        match self.status {
            ScheduledSinkPlanStatus::Planned => {
                return next == ScheduledSinkPlanStatus::Cancelled;
            }
            ScheduledSinkPlanStatus::Scheduled => {
                return next == ScheduledSinkPlanStatus::InProgress || next == ScheduledSinkPlanStatus::Cancelled;
            }
            ScheduledSinkPlanStatus::InProgress => {
                return next == ScheduledSinkPlanStatus::Cancelled;
            }
            ScheduledSinkPlanStatus::Completed => {
                return next == ScheduledSinkPlanStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledSinkPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled sink plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled sink plan", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.amount + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledSinkPlan {
        ScheduledSinkPlan {
            code: self.code.clone(),
            title: self.title.clone(),
            external_ref: self.external_ref.clone(),
            amount: self.amount,
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled sink plan in status planned.
pub fn new_scheduled_sink_plan(code: &str, title: &str) -> ScheduledSinkPlan {
    ScheduledSinkPlan {
        code: code.to_string(),
        title: title.to_string(),
        external_ref: "".to_string(),
        amount: 0.0,
        principal: "".to_string(),
        ttl_seconds: 0,
        status: ScheduledSinkPlanStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_sink_plan("code-813", "title-183");
        assert_eq!(item.status, ScheduledSinkPlanStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_scheduled_sink_plan("code-813", "title-183");
        if item.move_to(ScheduledSinkPlanStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledSinkPlanStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_sink_plan("code-813", "title-183");
        item.status = ScheduledSinkPlanStatus::Cancelled;
        assert!(
            item.move_to(ScheduledSinkPlanStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
