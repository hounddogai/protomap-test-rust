use super::*;

use acme_kit::text;

/// Where a scheduled limit cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledLimitCycleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ScheduledLimitCycleStatus {
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

impl std::fmt::Display for ScheduledLimitCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled limit cycle as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledLimitCycle {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub version: i64,
    pub burst: i64,
    pub status: ScheduledLimitCycleStatus,
}

impl ScheduledLimitCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledLimitCycleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledLimitCycleStatus) -> bool {
        match self.status {
            ScheduledLimitCycleStatus::Planned => {
                match next {
                    ScheduledLimitCycleStatus::InProgress | ScheduledLimitCycleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledLimitCycleStatus::Scheduled => {
                return next == ScheduledLimitCycleStatus::Cancelled;
            }
            ScheduledLimitCycleStatus::InProgress => {
                return next == ScheduledLimitCycleStatus::Completed;
            }
            ScheduledLimitCycleStatus::Completed => {
                return next == ScheduledLimitCycleStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledLimitCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled limit cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.burst as f64 + self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledLimitCycle {
        ScheduledLimitCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            version: self.version,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled limit cycle in status planned.
pub fn new_scheduled_limit_cycle(code: &str, name: &str) -> ScheduledLimitCycle {
    ScheduledLimitCycle {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        version: 0,
        burst: 0,
        status: ScheduledLimitCycleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_scheduled_limit_cycle("code-955", "name-836");
        assert_eq!(item.status, ScheduledLimitCycleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_scheduled_limit_cycle("code-955", "name-836");
        if item.move_to(ScheduledLimitCycleStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledLimitCycleStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_scheduled_limit_cycle("code-955", "name-836");
        item.status = ScheduledLimitCycleStatus::Cancelled;
        assert!(
            item.move_to(ScheduledLimitCycleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
