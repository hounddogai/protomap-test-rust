use super::*;

use acme_kit::text;

/// Where a seasonal allowlist milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalAllowlistMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalAllowlistMilestoneStatus {
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

impl std::fmt::Display for SeasonalAllowlistMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal allowlist milestone, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalAllowlistMilestone {
    pub code: String,
    pub label: String,
    pub limit: i64,
    pub ttl_seconds: i64,
    pub fingerprint: String,
    pub status: SeasonalAllowlistMilestoneStatus,
}

impl SeasonalAllowlistMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalAllowlistMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalAllowlistMilestoneStatus) -> bool {
        match self.status {
            SeasonalAllowlistMilestoneStatus::Idle => {
                return next == SeasonalAllowlistMilestoneStatus::Ready;
            }
            SeasonalAllowlistMilestoneStatus::Warming => {
                return next == SeasonalAllowlistMilestoneStatus::Draining;
            }
            SeasonalAllowlistMilestoneStatus::Ready => {
                match next {
                    SeasonalAllowlistMilestoneStatus::Draining | SeasonalAllowlistMilestoneStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalAllowlistMilestoneStatus::Draining => {
                match next {
                    SeasonalAllowlistMilestoneStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalAllowlistMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal allowlist milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.limit as f64) * (self.ttl_seconds as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalAllowlistMilestone {
        SeasonalAllowlistMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            limit: self.limit,
            ttl_seconds: self.ttl_seconds,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a seasonal allowlist milestone in status idle.
pub fn new_seasonal_allowlist_milestone(code: &str, label: &str) -> SeasonalAllowlistMilestone {
    SeasonalAllowlistMilestone {
        code: code.to_string(),
        label: label.to_string(),
        limit: 0,
        ttl_seconds: 0,
        fingerprint: "".to_string(),
        status: SeasonalAllowlistMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_allowlist_milestone("code-45", "label-622");
        assert_eq!(item.status, SeasonalAllowlistMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_seasonal_allowlist_milestone("code-45", "label-622");
        if item.move_to(SeasonalAllowlistMilestoneStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalAllowlistMilestoneStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_allowlist_milestone("code-45", "label-622");
        item.status = SeasonalAllowlistMilestoneStatus::Stopped;
        assert!(
            item.move_to(SeasonalAllowlistMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
