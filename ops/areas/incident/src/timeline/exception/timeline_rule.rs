use super::*;

use acme_kit::text;

/// Where a timeline rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TimelineRuleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TimelineRuleStatus {
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

impl std::fmt::Display for TimelineRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a timeline rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TimelineRule {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub description: String,
    pub issuer: String,
    pub window_days: i64,
    pub status: TimelineRuleStatus,
}

impl TimelineRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TimelineRuleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TimelineRuleStatus) -> bool {
        match self.status {
            TimelineRuleStatus::Idle => {
                return next == TimelineRuleStatus::Warming;
            }
            TimelineRuleStatus::Warming => {
                return next == TimelineRuleStatus::Ready || next == TimelineRuleStatus::Stopped;
            }
            TimelineRuleStatus::Ready => {
                match next {
                    TimelineRuleStatus::Draining | TimelineRuleStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TimelineRuleStatus::Draining => {
                return next == TimelineRuleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TimelineRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "timeline rule {} cannot move from {} to {}",
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
        self.window_days as f64
    }

    pub fn copy(&self) -> TimelineRule {
        TimelineRule {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            description: self.description.clone(),
            issuer: self.issuer.clone(),
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a timeline rule in status idle.
pub fn new_timeline_rule(code: &str, name: &str) -> TimelineRule {
    TimelineRule {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        description: "".to_string(),
        issuer: "".to_string(),
        window_days: 0,
        status: TimelineRuleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_timeline_rule("code-695", "name-571");
        assert_eq!(item.status, TimelineRuleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_timeline_rule("code-695", "name-571");
        if item.move_to(TimelineRuleStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TimelineRuleStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_timeline_rule("code-695", "name-571");
        item.status = TimelineRuleStatus::Stopped;
        assert!(item.move_to(TimelineRuleStatus::Idle).is_err(), "expected move to fail");
    }
}
