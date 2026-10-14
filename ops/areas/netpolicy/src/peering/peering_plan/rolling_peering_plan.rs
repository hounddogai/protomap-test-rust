use super::*;

use acme_kit::text;

/// Where a rolling peering plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingPeeringPlanStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl RollingPeeringPlanStatus {
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

impl std::fmt::Display for RollingPeeringPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling peering plan and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingPeeringPlan {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub reviewer: String,
    pub region: String,
    pub weight: f64,
    pub scope: String,
    pub grace_days: i64,
    pub locale: String,
    pub status: RollingPeeringPlanStatus,
}

impl RollingPeeringPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingPeeringPlanStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingPeeringPlanStatus) -> bool {
        match self.status {
            RollingPeeringPlanStatus::New => {
                match next {
                    RollingPeeringPlanStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingPeeringPlanStatus::Queued => {
                return next == RollingPeeringPlanStatus::Done;
            }
            RollingPeeringPlanStatus::Running => {
                return next == RollingPeeringPlanStatus::Done || next == RollingPeeringPlanStatus::Failed;
            }
            RollingPeeringPlanStatus::Done => {
                match next {
                    RollingPeeringPlanStatus::Failed => {
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

    pub fn move_to(&mut self, next: RollingPeeringPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling peering plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.grace_days as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingPeeringPlan {
        RollingPeeringPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            reviewer: self.reviewer.clone(),
            region: self.region.clone(),
            weight: self.weight,
            scope: self.scope.clone(),
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling peering plan in status new.
pub fn new_rolling_peering_plan(code: &str, label: &str) -> RollingPeeringPlan {
    RollingPeeringPlan {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        reviewer: "".to_string(),
        region: "".to_string(),
        weight: 0.0,
        scope: "".to_string(),
        grace_days: 0,
        locale: "".to_string(),
        status: RollingPeeringPlanStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_rolling_peering_plan("code-321", "label-241");
        assert_eq!(item.status, RollingPeeringPlanStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_rolling_peering_plan("code-321", "label-241");
        if item.move_to(RollingPeeringPlanStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingPeeringPlanStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_rolling_peering_plan("code-321", "label-241");
        item.status = RollingPeeringPlanStatus::Failed;
        assert!(
            item.move_to(RollingPeeringPlanStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
