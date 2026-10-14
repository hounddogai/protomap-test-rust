use super::*;

use acme_kit::text;

/// Where a failover profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl FailoverProfileStatus {
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

impl std::fmt::Display for FailoverProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One failover profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverProfile {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub contact_email: String,
    pub duration_minutes: i64,
    pub status: FailoverProfileStatus,
}

impl FailoverProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverProfileStatus) -> bool {
        match self.status {
            FailoverProfileStatus::Idle => {
                return next == FailoverProfileStatus::Stopped;
            }
            FailoverProfileStatus::Warming => {
                match next {
                    FailoverProfileStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverProfileStatus::Ready => {
                return next == FailoverProfileStatus::Draining || next == FailoverProfileStatus::Stopped;
            }
            FailoverProfileStatus::Draining => {
                match next {
                    FailoverProfileStatus::Stopped => {
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

    pub fn move_to(&mut self, next: FailoverProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "failover profile", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.grace_days as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverProfile {
        FailoverProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            contact_email: self.contact_email.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a failover profile in status idle.
pub fn new_failover_profile(code: &str, name: &str) -> FailoverProfile {
    FailoverProfile {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        contact_email: "".to_string(),
        duration_minutes: 0,
        status: FailoverProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_failover_profile("code-556", "name-853");
        assert_eq!(item.status, FailoverProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_failover_profile("code-556", "name-853");
        if item.move_to(FailoverProfileStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FailoverProfileStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_failover_profile("code-556", "name-853");
        item.status = FailoverProfileStatus::Stopped;
        assert!(
            item.move_to(FailoverProfileStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
