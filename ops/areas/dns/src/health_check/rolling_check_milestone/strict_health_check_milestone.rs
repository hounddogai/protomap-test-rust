use super::*;

use acme_kit::text;

/// Where a strict health check milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictHealthCheckMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictHealthCheckMilestoneStatus {
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

impl std::fmt::Display for StrictHealthCheckMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict health check milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictHealthCheckMilestone {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub namespace: String,
    pub margin: f64,
    pub status: StrictHealthCheckMilestoneStatus,
}

impl StrictHealthCheckMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictHealthCheckMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictHealthCheckMilestoneStatus) -> bool {
        match self.status {
            StrictHealthCheckMilestoneStatus::Idle => {
                match next {
                    StrictHealthCheckMilestoneStatus::Warming => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckMilestoneStatus::Warming => {
                return next == StrictHealthCheckMilestoneStatus::Draining;
            }
            StrictHealthCheckMilestoneStatus::Ready => {
                match next {
                    StrictHealthCheckMilestoneStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictHealthCheckMilestoneStatus::Draining => {
                match next {
                    StrictHealthCheckMilestoneStatus::Stopped => {
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

    pub fn move_to(&mut self, next: StrictHealthCheckMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict health check milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "strict health check milestone", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.margin
    }

    pub fn copy(&self) -> StrictHealthCheckMilestone {
        StrictHealthCheckMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            namespace: self.namespace.clone(),
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict health check milestone in status idle.
pub fn new_strict_health_check_milestone(code: &str, title: &str) -> StrictHealthCheckMilestone {
    StrictHealthCheckMilestone {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        namespace: "".to_string(),
        margin: 0.0,
        status: StrictHealthCheckMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_health_check_milestone("code-326", "title-802");
        assert_eq!(item.status, StrictHealthCheckMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_strict_health_check_milestone("code-326", "title-802");
        if item.move_to(StrictHealthCheckMilestoneStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictHealthCheckMilestoneStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_health_check_milestone("code-326", "title-802");
        item.status = StrictHealthCheckMilestoneStatus::Stopped;
        assert!(
            item.move_to(StrictHealthCheckMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
