use super::*;

use acme_kit::text;

/// Where a canary budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl CanaryBudgetStatus {
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

impl std::fmt::Display for CanaryBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a canary budget and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryBudget {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub host: String,
    pub rate: f64,
    pub reviewer: String,
    pub status: CanaryBudgetStatus,
}

impl CanaryBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryBudgetStatus) -> bool {
        match self.status {
            CanaryBudgetStatus::Idle => {
                match next {
                    CanaryBudgetStatus::Ready | CanaryBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryBudgetStatus::Warming => {
                match next {
                    CanaryBudgetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryBudgetStatus::Ready => {
                match next {
                    CanaryBudgetStatus::Draining | CanaryBudgetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryBudgetStatus::Draining => {
                return next == CanaryBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "canary budget", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.rate
    }

    pub fn copy(&self) -> CanaryBudget {
        CanaryBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            host: self.host.clone(),
            rate: self.rate,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a canary budget in status idle.
pub fn new_canary_budget(code: &str, name: &str) -> CanaryBudget {
    CanaryBudget {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        host: "".to_string(),
        rate: 0.0,
        reviewer: "".to_string(),
        status: CanaryBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_canary_budget("code-355", "name-57");
        assert_eq!(item.status, CanaryBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_canary_budget("code-355", "name-57");
        if item.move_to(CanaryBudgetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryBudgetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_canary_budget("code-355", "name-57");
        item.status = CanaryBudgetStatus::Stopped;
        assert!(item.move_to(CanaryBudgetStatus::Idle).is_err(), "expected move to fail");
    }
}
