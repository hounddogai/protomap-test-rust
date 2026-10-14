use super::*;

use acme_kit::text;

/// Where a capped token schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTokenScheduleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CappedTokenScheduleStatus {
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

impl std::fmt::Display for CappedTokenScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped token schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTokenSchedule {
    pub code: String,
    pub name: String,
    pub principal: String,
    pub duration_minutes: i64,
    pub host: String,
    pub tenant_id: String,
    pub reviewer: String,
    pub label: String,
    pub phone: String,
    pub status: CappedTokenScheduleStatus,
}

impl CappedTokenSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTokenScheduleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTokenScheduleStatus) -> bool {
        match self.status {
            CappedTokenScheduleStatus::Idle => {
                return next == CappedTokenScheduleStatus::Ready || next == CappedTokenScheduleStatus::Draining;
            }
            CappedTokenScheduleStatus::Warming => {
                return next == CappedTokenScheduleStatus::Stopped;
            }
            CappedTokenScheduleStatus::Ready => {
                return next == CappedTokenScheduleStatus::Stopped;
            }
            CappedTokenScheduleStatus::Draining => {
                match next {
                    CappedTokenScheduleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: CappedTokenScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped token schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> CappedTokenSchedule {
        CappedTokenSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            principal: self.principal.clone(),
            duration_minutes: self.duration_minutes,
            host: self.host.clone(),
            tenant_id: self.tenant_id.clone(),
            reviewer: self.reviewer.clone(),
            label: self.label.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a capped token schedule in status idle.
pub fn new_capped_token_schedule(code: &str, name: &str) -> CappedTokenSchedule {
    CappedTokenSchedule {
        code: code.to_string(),
        name: name.to_string(),
        principal: "".to_string(),
        duration_minutes: 0,
        host: "".to_string(),
        tenant_id: "".to_string(),
        reviewer: "".to_string(),
        label: "".to_string(),
        phone: "".to_string(),
        status: CappedTokenScheduleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_capped_token_schedule("code-468", "name-426");
        assert_eq!(item.status, CappedTokenScheduleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_capped_token_schedule("code-468", "name-426");
        if item.move_to(CappedTokenScheduleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTokenScheduleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_capped_token_schedule("code-468", "name-426");
        item.status = CappedTokenScheduleStatus::Stopped;
        assert!(
            item.move_to(CappedTokenScheduleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
