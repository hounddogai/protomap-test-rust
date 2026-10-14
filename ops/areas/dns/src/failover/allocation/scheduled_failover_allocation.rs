use super::*;

use acme_kit::text;

/// Where a scheduled failover allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFailoverAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledFailoverAllocationStatus {
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

impl std::fmt::Display for ScheduledFailoverAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled failover allocation as the failover team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFailoverAllocation {
    pub code: String,
    pub label: String,
    pub source: String,
    pub attempts: i64,
    pub minimum: f64,
    pub cost: f64,
    pub status: ScheduledFailoverAllocationStatus,
}

impl ScheduledFailoverAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFailoverAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFailoverAllocationStatus) -> bool {
        match self.status {
            ScheduledFailoverAllocationStatus::Idle => {
                return next == ScheduledFailoverAllocationStatus::Warming;
            }
            ScheduledFailoverAllocationStatus::Warming => {
                match next {
                    ScheduledFailoverAllocationStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFailoverAllocationStatus::Ready => {
                match next {
                    ScheduledFailoverAllocationStatus::Draining | ScheduledFailoverAllocationStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFailoverAllocationStatus::Draining => {
                return next == ScheduledFailoverAllocationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFailoverAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled failover allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "scheduled failover allocation", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.minimum * (self.attempts as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFailoverAllocation {
        ScheduledFailoverAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            attempts: self.attempts,
            minimum: self.minimum,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled failover allocation in status idle.
pub fn new_scheduled_failover_allocation(code: &str, label: &str) -> ScheduledFailoverAllocation {
    ScheduledFailoverAllocation {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        attempts: 0,
        minimum: 0.0,
        cost: 0.0,
        status: ScheduledFailoverAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_failover_allocation("code-190", "label-50");
        assert_eq!(item.status, ScheduledFailoverAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_scheduled_failover_allocation("code-190", "label-50");
        if item.move_to(ScheduledFailoverAllocationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFailoverAllocationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_failover_allocation("code-190", "label-50");
        item.status = ScheduledFailoverAllocationStatus::Stopped;
        assert!(
            item.move_to(ScheduledFailoverAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
