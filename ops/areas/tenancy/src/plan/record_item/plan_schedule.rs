use super::*;

use acme_kit::text;

/// Where a plan schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanScheduleStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl PlanScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for PlanScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanSchedule {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub ttl_seconds: i64,
    pub rotation_days: i64,
    pub description: String,
    pub active: bool,
    pub ceiling: f64,
    pub ratio: f64,
    pub status: PlanScheduleStatus,
}

impl PlanSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanScheduleStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanScheduleStatus) -> bool {
        match self.status {
            PlanScheduleStatus::New => {
                match next {
                    PlanScheduleStatus::Queued | PlanScheduleStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanScheduleStatus::Queued => {
                return next == PlanScheduleStatus::Running;
            }
            PlanScheduleStatus::Running => {
                match next {
                    PlanScheduleStatus::Done | PlanScheduleStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanScheduleStatus::Done => {
                match next {
                    PlanScheduleStatus::Failed => {
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

    pub fn move_to(&mut self, next: PlanScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64) * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanSchedule {
        PlanSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            ttl_seconds: self.ttl_seconds,
            rotation_days: self.rotation_days,
            description: self.description.clone(),
            active: self.active,
            ceiling: self.ceiling,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a plan schedule in status new.
pub fn new_plan_schedule(code: &str, name: &str) -> PlanSchedule {
    PlanSchedule {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        ttl_seconds: 0,
        rotation_days: 0,
        description: "".to_string(),
        active: false,
        ceiling: 0.0,
        ratio: 0.0,
        status: PlanScheduleStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_plan_schedule("code-171", "name-949");
        assert_eq!(item.status, PlanScheduleStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_plan_schedule("code-171", "name-949");
        if item.move_to(PlanScheduleStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanScheduleStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_plan_schedule("code-171", "name-949");
        item.status = PlanScheduleStatus::Failed;
        assert!(item.move_to(PlanScheduleStatus::New).is_err(), "expected move to fail");
    }
}
