use super::*;

use acme_kit::text;

/// Where a rollback baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackBaselineStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollbackBaselineStatus {
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

impl std::fmt::Display for RollbackBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rollback baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackBaseline {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub sequence: i64,
    pub summary: String,
    pub burst: i64,
    pub status: RollbackBaselineStatus,
}

impl RollbackBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackBaselineStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackBaselineStatus) -> bool {
        match self.status {
            RollbackBaselineStatus::Idle => {
                return next == RollbackBaselineStatus::Warming;
            }
            RollbackBaselineStatus::Warming => {
                match next {
                    RollbackBaselineStatus::Draining | RollbackBaselineStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollbackBaselineStatus::Ready => {
                return next == RollbackBaselineStatus::Stopped;
            }
            RollbackBaselineStatus::Draining => {
                return next == RollbackBaselineStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollbackBaseline {
        RollbackBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            sequence: self.sequence,
            summary: self.summary.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a rollback baseline in status idle.
pub fn new_rollback_baseline(code: &str, title: &str) -> RollbackBaseline {
    RollbackBaseline {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        sequence: 0,
        summary: "".to_string(),
        burst: 0,
        status: RollbackBaselineStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rollback_baseline("code-964", "title-220");
        assert_eq!(item.status, RollbackBaselineStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_rollback_baseline("code-964", "title-220");
        if item.move_to(RollbackBaselineStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackBaselineStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rollback_baseline("code-964", "title-220");
        item.status = RollbackBaselineStatus::Stopped;
        assert!(
            item.move_to(RollbackBaselineStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
