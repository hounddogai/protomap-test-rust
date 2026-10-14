use super::*;

use acme_kit::text;

/// Where a rule window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RuleWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RuleWindowStatus {
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

impl std::fmt::Display for RuleWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rule window and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RuleWindow {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub budget: f64,
    pub cluster: String,
    pub category: String,
    pub amount: f64,
    pub burst: i64,
    pub status: RuleWindowStatus,
}

impl RuleWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RuleWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RuleWindowStatus) -> bool {
        match self.status {
            RuleWindowStatus::Idle => {
                match next {
                    RuleWindowStatus::Ready | RuleWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleWindowStatus::Warming => {
                return next == RuleWindowStatus::Stopped;
            }
            RuleWindowStatus::Ready => {
                match next {
                    RuleWindowStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RuleWindowStatus::Draining => {
                return next == RuleWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RuleWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rule window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.burst as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RuleWindow {
        RuleWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            budget: self.budget,
            cluster: self.cluster.clone(),
            category: self.category.clone(),
            amount: self.amount,
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a rule window in status idle.
pub fn new_rule_window(code: &str, label: &str) -> RuleWindow {
    RuleWindow {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        budget: 0.0,
        cluster: "".to_string(),
        category: "".to_string(),
        amount: 0.0,
        burst: 0,
        status: RuleWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rule_window("code-664", "label-686");
        assert_eq!(item.status, RuleWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_rule_window("code-664", "label-686");
        if item.move_to(RuleWindowStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RuleWindowStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rule_window("code-664", "label-686");
        item.status = RuleWindowStatus::Stopped;
        assert!(item.move_to(RuleWindowStatus::Idle).is_err(), "expected move to fail");
    }
}
