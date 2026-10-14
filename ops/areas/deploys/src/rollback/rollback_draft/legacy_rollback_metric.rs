use super::*;

use acme_kit::text;

/// Where a legacy rollback metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyRollbackMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyRollbackMetricStatus {
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

impl std::fmt::Display for LegacyRollbackMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy rollback metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyRollbackMetric {
    pub code: String,
    pub name: String,
    pub namespace: String,
    pub title: String,
    pub attempts: i64,
    pub status: LegacyRollbackMetricStatus,
}

impl LegacyRollbackMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyRollbackMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyRollbackMetricStatus) -> bool {
        match self.status {
            LegacyRollbackMetricStatus::Idle => {
                match next {
                    LegacyRollbackMetricStatus::Warming | LegacyRollbackMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackMetricStatus::Warming => {
                match next {
                    LegacyRollbackMetricStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyRollbackMetricStatus::Ready => {
                return next == LegacyRollbackMetricStatus::Draining || next == LegacyRollbackMetricStatus::Stopped;
            }
            LegacyRollbackMetricStatus::Draining => {
                return next == LegacyRollbackMetricStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyRollbackMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy rollback metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> LegacyRollbackMetric {
        LegacyRollbackMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            namespace: self.namespace.clone(),
            title: self.title.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a legacy rollback metric in status idle.
pub fn new_legacy_rollback_metric(code: &str, name: &str) -> LegacyRollbackMetric {
    LegacyRollbackMetric {
        code: code.to_string(),
        name: name.to_string(),
        namespace: "".to_string(),
        title: "".to_string(),
        attempts: 0,
        status: LegacyRollbackMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_legacy_rollback_metric("code-225", "name-409");
        assert_eq!(item.status, LegacyRollbackMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_legacy_rollback_metric("code-225", "name-409");
        if item.move_to(LegacyRollbackMetricStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyRollbackMetricStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_legacy_rollback_metric("code-225", "name-409");
        item.status = LegacyRollbackMetricStatus::Stopped;
        assert!(
            item.move_to(LegacyRollbackMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
