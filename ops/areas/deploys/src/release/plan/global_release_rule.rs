use super::*;

use acme_kit::text;

/// Where a global release rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleaseRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalReleaseRuleStatus {
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

impl std::fmt::Display for GlobalReleaseRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A global release rule as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseRule {
    pub code: String,
    pub name: String,
    pub revision: i64,
    pub threshold: f64,
    pub reviewer: String,
    pub timezone: String,
    pub burst: i64,
    pub source: String,
    pub status: GlobalReleaseRuleStatus,
}

impl GlobalReleaseRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleaseRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleaseRuleStatus) -> bool {
        match self.status {
            GlobalReleaseRuleStatus::Idle => {
                match next {
                    GlobalReleaseRuleStatus::Warming | GlobalReleaseRuleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseRuleStatus::Warming => {
                match next {
                    GlobalReleaseRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalReleaseRuleStatus::Ready => {
                return next == GlobalReleaseRuleStatus::Draining;
            }
            GlobalReleaseRuleStatus::Draining => {
                match next {
                    GlobalReleaseRuleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalReleaseRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.threshold * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReleaseRule {
        GlobalReleaseRule {
            code: self.code.clone(),
            name: self.name.clone(),
            revision: self.revision,
            threshold: self.threshold,
            reviewer: self.reviewer.clone(),
            timezone: self.timezone.clone(),
            burst: self.burst,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a global release rule in status idle.
pub fn new_global_release_rule(code: &str, name: &str) -> GlobalReleaseRule {
    GlobalReleaseRule {
        code: code.to_string(),
        name: name.to_string(),
        revision: 0,
        threshold: 0.0,
        reviewer: "".to_string(),
        timezone: "".to_string(),
        burst: 0,
        source: "".to_string(),
        status: GlobalReleaseRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_release_rule("code-678", "name-537");
        assert_eq!(item.status, GlobalReleaseRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_global_release_rule("code-678", "name-537");
        if item.move_to(GlobalReleaseRuleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalReleaseRuleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_release_rule("code-678", "name-537");
        item.status = GlobalReleaseRuleStatus::Stopped;
        assert!(
            item.move_to(GlobalReleaseRuleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
