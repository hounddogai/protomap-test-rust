use super::*;

use acme_kit::text;

/// Where a primary plan schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPlanScheduleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryPlanScheduleStatus {
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

impl std::fmt::Display for PrimaryPlanScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary plan schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanSchedule {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub ttl_seconds: i64,
    pub grace_days: i64,
    pub description: String,
    pub priority: i64,
    pub tenant_id: String,
    pub pinned: bool,
    pub status: PrimaryPlanScheduleStatus,
}

impl PrimaryPlanSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPlanScheduleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPlanScheduleStatus) -> bool {
        match self.status {
            PrimaryPlanScheduleStatus::Planned => {
                return next == PrimaryPlanScheduleStatus::Cancelled;
            }
            PrimaryPlanScheduleStatus::Scheduled => {
                return next == PrimaryPlanScheduleStatus::InProgress || next == PrimaryPlanScheduleStatus::Cancelled;
            }
            PrimaryPlanScheduleStatus::InProgress => {
                return next == PrimaryPlanScheduleStatus::Completed || next == PrimaryPlanScheduleStatus::Cancelled;
            }
            PrimaryPlanScheduleStatus::Completed => {
                return next == PrimaryPlanScheduleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPlanScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary plan schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.quota_units as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPlanSchedule {
        PrimaryPlanSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            ttl_seconds: self.ttl_seconds,
            grace_days: self.grace_days,
            description: self.description.clone(),
            priority: self.priority,
            tenant_id: self.tenant_id.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a primary plan schedule in status planned.
pub fn new_primary_plan_schedule(code: &str, name: &str) -> PrimaryPlanSchedule {
    PrimaryPlanSchedule {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        ttl_seconds: 0,
        grace_days: 0,
        description: "".to_string(),
        priority: 0,
        tenant_id: "".to_string(),
        pinned: false,
        status: PrimaryPlanScheduleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_plan_schedule("code-598", "name-593");
        assert_eq!(item.status, PrimaryPlanScheduleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_primary_plan_schedule("code-598", "name-593");
        if item.move_to(PrimaryPlanScheduleStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPlanScheduleStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_plan_schedule("code-598", "name-593");
        item.status = PrimaryPlanScheduleStatus::Cancelled;
        assert!(
            item.move_to(PrimaryPlanScheduleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
