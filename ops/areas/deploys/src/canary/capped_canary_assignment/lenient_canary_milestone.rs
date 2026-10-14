use super::*;

use acme_kit::text;

/// Where a lenient canary milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCanaryMilestoneStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientCanaryMilestoneStatus {
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

impl std::fmt::Display for LenientCanaryMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient canary milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryMilestone {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub budget: f64,
    pub attempts: i64,
    pub cluster: String,
    pub status: LenientCanaryMilestoneStatus,
}

impl LenientCanaryMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCanaryMilestoneStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCanaryMilestoneStatus) -> bool {
        match self.status {
            LenientCanaryMilestoneStatus::Idle => {
                match next {
                    LenientCanaryMilestoneStatus::Ready | LenientCanaryMilestoneStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCanaryMilestoneStatus::Warming => {
                return next == LenientCanaryMilestoneStatus::Ready || next == LenientCanaryMilestoneStatus::Draining;
            }
            LenientCanaryMilestoneStatus::Ready => {
                return next == LenientCanaryMilestoneStatus::Stopped;
            }
            LenientCanaryMilestoneStatus::Draining => {
                return next == LenientCanaryMilestoneStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientCanaryMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient canary milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.budget - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCanaryMilestone {
        LenientCanaryMilestone {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            budget: self.budget,
            attempts: self.attempts,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a lenient canary milestone in status idle.
pub fn new_lenient_canary_milestone(code: &str, label: &str) -> LenientCanaryMilestone {
    LenientCanaryMilestone {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        budget: 0.0,
        attempts: 0,
        cluster: "".to_string(),
        status: LenientCanaryMilestoneStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_canary_milestone("code-917", "label-584");
        assert_eq!(item.status, LenientCanaryMilestoneStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_lenient_canary_milestone("code-917", "label-584");
        if item.move_to(LenientCanaryMilestoneStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientCanaryMilestoneStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_canary_milestone("code-917", "label-584");
        item.status = LenientCanaryMilestoneStatus::Stopped;
        assert!(
            item.move_to(LenientCanaryMilestoneStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
