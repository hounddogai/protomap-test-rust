use super::*;

use acme_kit::text;

/// Where a global segment plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalSegmentPlanStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalSegmentPlanStatus {
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

impl std::fmt::Display for GlobalSegmentPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global segment plan, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalSegmentPlan {
    pub code: String,
    pub label: String,
    pub unit: String,
    pub source: String,
    pub margin: f64,
    pub floor_value: f64,
    pub ends_at: i64,
    pub priority: i64,
    pub status: GlobalSegmentPlanStatus,
}

impl GlobalSegmentPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalSegmentPlanStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalSegmentPlanStatus) -> bool {
        match self.status {
            GlobalSegmentPlanStatus::Idle => {
                return next == GlobalSegmentPlanStatus::Warming || next == GlobalSegmentPlanStatus::Draining;
            }
            GlobalSegmentPlanStatus::Warming => {
                match next {
                    GlobalSegmentPlanStatus::Draining | GlobalSegmentPlanStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentPlanStatus::Ready => {
                match next {
                    GlobalSegmentPlanStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalSegmentPlanStatus::Draining => {
                return next == GlobalSegmentPlanStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalSegmentPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global segment plan {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "global segment plan", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.ends_at as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalSegmentPlan {
        GlobalSegmentPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            unit: self.unit.clone(),
            source: self.source.clone(),
            margin: self.margin,
            floor_value: self.floor_value,
            ends_at: self.ends_at,
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a global segment plan in status idle.
pub fn new_global_segment_plan(code: &str, label: &str) -> GlobalSegmentPlan {
    GlobalSegmentPlan {
        code: code.to_string(),
        label: label.to_string(),
        unit: "".to_string(),
        source: "".to_string(),
        margin: 0.0,
        floor_value: 0.0,
        ends_at: 0,
        priority: 0,
        status: GlobalSegmentPlanStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_segment_plan("code-824", "label-192");
        assert_eq!(item.status, GlobalSegmentPlanStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_global_segment_plan("code-824", "label-192");
        if item.move_to(GlobalSegmentPlanStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalSegmentPlanStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_segment_plan("code-824", "label-192");
        item.status = GlobalSegmentPlanStatus::Stopped;
        assert!(
            item.move_to(GlobalSegmentPlanStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
