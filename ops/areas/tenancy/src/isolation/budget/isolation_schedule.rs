use super::*;

use acme_kit::text;

/// Where a isolation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IsolationScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl IsolationScheduleStatus {
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

impl std::fmt::Display for IsolationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One isolation schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct IsolationSchedule {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub retries: i64,
    pub grace_days: i64,
    pub rate: f64,
    pub amount: f64,
    pub title: String,
    pub principal: String,
    pub status: IsolationScheduleStatus,
}

impl IsolationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IsolationScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IsolationScheduleStatus) -> bool {
        match self.status {
            IsolationScheduleStatus::Idle => {
                match next {
                    IsolationScheduleStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationScheduleStatus::Warming => {
                match next {
                    IsolationScheduleStatus::Ready | IsolationScheduleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IsolationScheduleStatus::Ready => {
                return next == IsolationScheduleStatus::Draining;
            }
            IsolationScheduleStatus::Draining => {
                match next {
                    IsolationScheduleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: IsolationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "isolation schedule {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_value(&self) -> f64 {
        (self.amount * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IsolationSchedule {
        IsolationSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            retries: self.retries,
            grace_days: self.grace_days,
            rate: self.rate,
            amount: self.amount,
            title: self.title.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a isolation schedule in status idle.
pub fn new_isolation_schedule(code: &str, name: &str) -> IsolationSchedule {
    IsolationSchedule {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        retries: 0,
        grace_days: 0,
        rate: 0.0,
        amount: 0.0,
        title: "".to_string(),
        principal: "".to_string(),
        status: IsolationScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_isolation_schedule("code-93", "name-427");
        assert_eq!(item.status, IsolationScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_isolation_schedule("code-93", "name-427");
        if item.move_to(IsolationScheduleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IsolationScheduleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_isolation_schedule("code-93", "name-427");
        item.status = IsolationScheduleStatus::Stopped;
        assert!(
            item.move_to(IsolationScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
