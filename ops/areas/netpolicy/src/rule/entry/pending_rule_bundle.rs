use super::*;

use acme_kit::text;

/// Where a pending rule bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRuleBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PendingRuleBundleStatus {
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

impl std::fmt::Display for PendingRuleBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending rule bundle as the rule team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRuleBundle {
    pub code: String,
    pub title: String,
    pub name: String,
    pub quota_units: i64,
    pub ratio: f64,
    pub status: PendingRuleBundleStatus,
}

impl PendingRuleBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRuleBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRuleBundleStatus) -> bool {
        match self.status {
            PendingRuleBundleStatus::Idle => {
                return next == PendingRuleBundleStatus::Ready;
            }
            PendingRuleBundleStatus::Warming => {
                return next == PendingRuleBundleStatus::Draining || next == PendingRuleBundleStatus::Stopped;
            }
            PendingRuleBundleStatus::Ready => {
                return next == PendingRuleBundleStatus::Draining || next == PendingRuleBundleStatus::Stopped;
            }
            PendingRuleBundleStatus::Draining => {
                match next {
                    PendingRuleBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: PendingRuleBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending rule bundle {} cannot move from {} to {}",
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
        ((self.quota_units as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRuleBundle {
        PendingRuleBundle {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending rule bundle in status idle.
pub fn new_pending_rule_bundle(code: &str, title: &str) -> PendingRuleBundle {
    PendingRuleBundle {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        quota_units: 0,
        ratio: 0.0,
        status: PendingRuleBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_pending_rule_bundle("code-554", "title-383");
        assert_eq!(item.status, PendingRuleBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_pending_rule_bundle("code-554", "title-383");
        if item.move_to(PendingRuleBundleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingRuleBundleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_pending_rule_bundle("code-554", "title-383");
        item.status = PendingRuleBundleStatus::Stopped;
        assert!(
            item.move_to(PendingRuleBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
