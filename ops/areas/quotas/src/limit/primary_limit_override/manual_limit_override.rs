use super::*;

use acme_kit::text;

/// Where a manual limit override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualLimitOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualLimitOverrideStatus {
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

impl std::fmt::Display for ManualLimitOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual limit override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualLimitOverride {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub budget: f64,
    pub quota_units: i64,
    pub ceiling: f64,
    pub grace_days: i64,
    pub status: ManualLimitOverrideStatus,
}

impl ManualLimitOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualLimitOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualLimitOverrideStatus) -> bool {
        match self.status {
            ManualLimitOverrideStatus::Idle => {
                match next {
                    ManualLimitOverrideStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualLimitOverrideStatus::Warming => {
                match next {
                    ManualLimitOverrideStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualLimitOverrideStatus::Ready => {
                return next == ManualLimitOverrideStatus::Draining || next == ManualLimitOverrideStatus::Stopped;
            }
            ManualLimitOverrideStatus::Draining => {
                return next == ManualLimitOverrideStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualLimitOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual limit override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.budget + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualLimitOverride {
        ManualLimitOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            budget: self.budget,
            quota_units: self.quota_units,
            ceiling: self.ceiling,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a manual limit override in status idle.
pub fn new_manual_limit_override(code: &str, name: &str) -> ManualLimitOverride {
    ManualLimitOverride {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        budget: 0.0,
        quota_units: 0,
        ceiling: 0.0,
        grace_days: 0,
        status: ManualLimitOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_limit_override("code-600", "name-335");
        assert_eq!(item.status, ManualLimitOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_limit_override("code-600", "name-335");
        if item.move_to(ManualLimitOverrideStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualLimitOverrideStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_limit_override("code-600", "name-335");
        item.status = ManualLimitOverrideStatus::Stopped;
        assert!(
            item.move_to(ManualLimitOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
