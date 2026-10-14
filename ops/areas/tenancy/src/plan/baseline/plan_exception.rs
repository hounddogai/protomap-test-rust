use super::*;

use acme_kit::text;

/// Where a plan exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanExceptionStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PlanExceptionStatus {
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

impl std::fmt::Display for PlanExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan exception, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanException {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub issuer: String,
    pub country: String,
    pub created_at: i64,
    pub window_days: i64,
    pub duration_minutes: i64,
    pub status: PlanExceptionStatus,
}

impl PlanException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanExceptionStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanExceptionStatus) -> bool {
        match self.status {
            PlanExceptionStatus::Idle => {
                match next {
                    PlanExceptionStatus::Ready | PlanExceptionStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanExceptionStatus::Warming => {
                return next == PlanExceptionStatus::Ready || next == PlanExceptionStatus::Draining;
            }
            PlanExceptionStatus::Ready => {
                return next == PlanExceptionStatus::Stopped;
            }
            PlanExceptionStatus::Draining => {
                match next {
                    PlanExceptionStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PlanExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.duration_minutes as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanException {
        PlanException {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            issuer: self.issuer.clone(),
            country: self.country.clone(),
            created_at: self.created_at,
            window_days: self.window_days,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a plan exception in status idle.
pub fn new_plan_exception(code: &str, name: &str) -> PlanException {
    PlanException {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        issuer: "".to_string(),
        country: "".to_string(),
        created_at: 0,
        window_days: 0,
        duration_minutes: 0,
        status: PlanExceptionStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_plan_exception("code-303", "name-982");
        assert_eq!(item.status, PlanExceptionStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_plan_exception("code-303", "name-982");
        if item.move_to(PlanExceptionStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanExceptionStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_plan_exception("code-303", "name-982");
        item.status = PlanExceptionStatus::Stopped;
        assert!(
            item.move_to(PlanExceptionStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
