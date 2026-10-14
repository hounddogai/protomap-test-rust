use super::*;

use acme_kit::text;

/// Where a seasonal egress plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalEgressPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalEgressPlanStatus {
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

impl std::fmt::Display for SeasonalEgressPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A seasonal egress plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalEgressPlan {
    pub code: String,
    pub name: String,
    pub duration_minutes: i64,
    pub unit: String,
    pub floor_value: f64,
    pub key_id: String,
    pub ends_at: i64,
    pub status: SeasonalEgressPlanStatus,
}

impl SeasonalEgressPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalEgressPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalEgressPlanStatus) -> bool {
        match self.status {
            SeasonalEgressPlanStatus::Idle => {
                return next == SeasonalEgressPlanStatus::Draining || next == SeasonalEgressPlanStatus::Stopped;
            }
            SeasonalEgressPlanStatus::Warming => {
                return next == SeasonalEgressPlanStatus::Ready || next == SeasonalEgressPlanStatus::Stopped;
            }
            SeasonalEgressPlanStatus::Ready => {
                match next {
                    SeasonalEgressPlanStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalEgressPlanStatus::Draining => {
                match next {
                    SeasonalEgressPlanStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalEgressPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal egress plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "seasonal egress plan", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.floor_value * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalEgressPlan {
        SeasonalEgressPlan {
            code: self.code.clone(),
            name: self.name.clone(),
            duration_minutes: self.duration_minutes,
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            key_id: self.key_id.clone(),
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal egress plan in status idle.
pub fn new_seasonal_egress_plan(code: &str, name: &str) -> SeasonalEgressPlan {
    SeasonalEgressPlan {
        code: code.to_string(),
        name: name.to_string(),
        duration_minutes: 0,
        unit: "".to_string(),
        floor_value: 0.0,
        key_id: "".to_string(),
        ends_at: 0,
        status: SeasonalEgressPlanStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_egress_plan("code-714", "name-357");
        assert_eq!(item.status, SeasonalEgressPlanStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_seasonal_egress_plan("code-714", "name-357");
        if item.move_to(SeasonalEgressPlanStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalEgressPlanStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_egress_plan("code-714", "name-357");
        item.status = SeasonalEgressPlanStatus::Stopped;
        assert!(
            item.move_to(SeasonalEgressPlanStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
