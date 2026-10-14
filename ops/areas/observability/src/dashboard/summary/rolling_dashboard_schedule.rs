use super::*;

use acme_kit::text;

/// Where a rolling dashboard schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingDashboardScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingDashboardScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingDashboardScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling dashboard schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingDashboardSchedule {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub batch_size: i64,
    pub budget: f64,
    pub revision: i64,
    pub notes: String,
    pub floor_value: f64,
    pub status: RollingDashboardScheduleStatus,
}

impl RollingDashboardSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingDashboardScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingDashboardScheduleStatus) -> bool {
        match self.status {
            RollingDashboardScheduleStatus::Proposed => {
                return next == RollingDashboardScheduleStatus::Rejected
                    || next == RollingDashboardScheduleStatus::Withdrawn;
            }
            RollingDashboardScheduleStatus::Approved => {
                match next {
                    RollingDashboardScheduleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingDashboardScheduleStatus::Rejected => {
                match next {
                    RollingDashboardScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: RollingDashboardScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling dashboard schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling dashboard schedule", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.batch_size as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingDashboardSchedule {
        RollingDashboardSchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            batch_size: self.batch_size,
            budget: self.budget,
            revision: self.revision,
            notes: self.notes.clone(),
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling dashboard schedule in status proposed.
pub fn new_rolling_dashboard_schedule(code: &str, label: &str) -> RollingDashboardSchedule {
    RollingDashboardSchedule {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        batch_size: 0,
        budget: 0.0,
        revision: 0,
        notes: "".to_string(),
        floor_value: 0.0,
        status: RollingDashboardScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_dashboard_schedule("code-254", "label-216");
        assert_eq!(item.status, RollingDashboardScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_rolling_dashboard_schedule("code-254", "label-216");
        if item.move_to(RollingDashboardScheduleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingDashboardScheduleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_dashboard_schedule("code-254", "label-216");
        item.status = RollingDashboardScheduleStatus::Withdrawn;
        assert!(
            item.move_to(RollingDashboardScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
