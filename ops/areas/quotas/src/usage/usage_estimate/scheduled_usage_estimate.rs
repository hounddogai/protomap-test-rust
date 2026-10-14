use super::*;

use acme_kit::text;

/// Where a scheduled usage estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledUsageEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledUsageEstimateStatus {
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

impl std::fmt::Display for ScheduledUsageEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled usage estimate. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledUsageEstimate {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub category: String,
    pub timezone: String,
    pub retries: i64,
    pub status: ScheduledUsageEstimateStatus,
}

impl ScheduledUsageEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledUsageEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledUsageEstimateStatus) -> bool {
        match self.status {
            ScheduledUsageEstimateStatus::Idle => {
                match next {
                    ScheduledUsageEstimateStatus::Warming | ScheduledUsageEstimateStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledUsageEstimateStatus::Warming => {
                return next == ScheduledUsageEstimateStatus::Stopped;
            }
            ScheduledUsageEstimateStatus::Ready => {
                match next {
                    ScheduledUsageEstimateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledUsageEstimateStatus::Draining => {
                return next == ScheduledUsageEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledUsageEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled usage estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> ScheduledUsageEstimate {
        ScheduledUsageEstimate {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            category: self.category.clone(),
            timezone: self.timezone.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a scheduled usage estimate in status idle.
pub fn new_scheduled_usage_estimate(code: &str, name: &str) -> ScheduledUsageEstimate {
    ScheduledUsageEstimate {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        category: "".to_string(),
        timezone: "".to_string(),
        retries: 0,
        status: ScheduledUsageEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_usage_estimate("code-222", "name-295");
        assert_eq!(item.status, ScheduledUsageEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_usage_estimate("code-222", "name-295");
        if item.move_to(ScheduledUsageEstimateStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledUsageEstimateStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_usage_estimate("code-222", "name-295");
        item.status = ScheduledUsageEstimateStatus::Stopped;
        assert!(
            item.move_to(ScheduledUsageEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
