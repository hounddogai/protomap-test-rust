use super::*;

use acme_kit::text;

/// Where a dashboard schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DashboardScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DashboardScheduleStatus {
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

impl std::fmt::Display for DashboardScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One dashboard schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DashboardSchedule {
    pub code: String,
    pub name: String,
    pub limit: i64,
    pub country: String,
    pub batch_size: i64,
    pub status: DashboardScheduleStatus,
}

impl DashboardSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DashboardScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DashboardScheduleStatus) -> bool {
        match self.status {
            DashboardScheduleStatus::Idle => {
                match next {
                    DashboardScheduleStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DashboardScheduleStatus::Warming => {
                return next == DashboardScheduleStatus::Ready;
            }
            DashboardScheduleStatus::Ready => {
                return next == DashboardScheduleStatus::Draining;
            }
            DashboardScheduleStatus::Draining => {
                return next == DashboardScheduleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DashboardScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dashboard schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.limit as f64) * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DashboardSchedule {
        DashboardSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            limit: self.limit,
            country: self.country.clone(),
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a dashboard schedule in status idle.
pub fn new_dashboard_schedule(code: &str, name: &str) -> DashboardSchedule {
    DashboardSchedule {
        code: code.to_string(),
        name: name.to_string(),
        limit: 0,
        country: "".to_string(),
        batch_size: 0,
        status: DashboardScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_dashboard_schedule("code-329", "name-505");
        assert_eq!(item.status, DashboardScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_dashboard_schedule("code-329", "name-505");
        if item.move_to(DashboardScheduleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DashboardScheduleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_dashboard_schedule("code-329", "name-505");
        item.status = DashboardScheduleStatus::Stopped;
        assert!(
            item.move_to(DashboardScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
