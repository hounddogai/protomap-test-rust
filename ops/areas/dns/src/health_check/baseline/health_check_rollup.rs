use super::*;

use acme_kit::text;

/// Where a health check rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckRollupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl HealthCheckRollupStatus {
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

impl std::fmt::Display for HealthCheckRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check rollup, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckRollup {
    pub code: String,
    pub name: String,
    pub reviewer: String,
    pub external_ref: String,
    pub archived: bool,
    pub grace_days: i64,
    pub maximum: f64,
    pub summary: String,
    pub status: HealthCheckRollupStatus,
}

impl HealthCheckRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckRollupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckRollupStatus) -> bool {
        match self.status {
            HealthCheckRollupStatus::Idle => {
                match next {
                    HealthCheckRollupStatus::Ready | HealthCheckRollupStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckRollupStatus::Warming => {
                match next {
                    HealthCheckRollupStatus::Draining | HealthCheckRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckRollupStatus::Ready => {
                match next {
                    HealthCheckRollupStatus::Draining | HealthCheckRollupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckRollupStatus::Draining => {
                match next {
                    HealthCheckRollupStatus::Stopped => {
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

    pub fn move_to(&mut self, next: HealthCheckRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.grace_days as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckRollup {
        HealthCheckRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            external_ref: self.external_ref.clone(),
            archived: self.archived,
            grace_days: self.grace_days,
            maximum: self.maximum,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a health check rollup in status idle.
pub fn new_health_check_rollup(code: &str, name: &str) -> HealthCheckRollup {
    HealthCheckRollup {
        code: code.to_string(),
        name: name.to_string(),
        reviewer: "".to_string(),
        external_ref: "".to_string(),
        archived: false,
        grace_days: 0,
        maximum: 0.0,
        summary: "".to_string(),
        status: HealthCheckRollupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_health_check_rollup("code-828", "name-628");
        assert_eq!(item.status, HealthCheckRollupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_health_check_rollup("code-828", "name-628");
        if item.move_to(HealthCheckRollupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckRollupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_health_check_rollup("code-828", "name-628");
        item.status = HealthCheckRollupStatus::Stopped;
        assert!(
            item.move_to(HealthCheckRollupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
