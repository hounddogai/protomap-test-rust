use super::*;

use acme_kit::text;

/// Where a strict delegation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictDelegationBudgetStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl StrictDelegationBudgetStatus {
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

impl std::fmt::Display for StrictDelegationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One strict delegation budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictDelegationBudget {
    pub code: String,
    pub name: String,
    pub title: String,
    pub locale: String,
    pub ceiling: f64,
    pub floor_value: f64,
    pub status: StrictDelegationBudgetStatus,
}

impl StrictDelegationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictDelegationBudgetStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictDelegationBudgetStatus) -> bool {
        match self.status {
            StrictDelegationBudgetStatus::Idle => {
                match next {
                    StrictDelegationBudgetStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationBudgetStatus::Warming => {
                match next {
                    StrictDelegationBudgetStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationBudgetStatus::Ready => {
                match next {
                    StrictDelegationBudgetStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictDelegationBudgetStatus::Draining => {
                return next == StrictDelegationBudgetStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictDelegationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict delegation budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.ceiling - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictDelegationBudget {
        StrictDelegationBudget {
            code: self.code.clone(),
            name: self.name.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            ceiling: self.ceiling,
            floor_value: self.floor_value,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a strict delegation budget in status idle.
pub fn new_strict_delegation_budget(code: &str, name: &str) -> StrictDelegationBudget {
    StrictDelegationBudget {
        code: code.to_string(),
        name: name.to_string(),
        title: "".to_string(),
        locale: "".to_string(),
        ceiling: 0.0,
        floor_value: 0.0,
        status: StrictDelegationBudgetStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_strict_delegation_budget("code-871", "name-658");
        assert_eq!(item.status, StrictDelegationBudgetStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_strict_delegation_budget("code-871", "name-658");
        if item.move_to(StrictDelegationBudgetStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictDelegationBudgetStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_strict_delegation_budget("code-871", "name-658");
        item.status = StrictDelegationBudgetStatus::Stopped;
        assert!(
            item.move_to(StrictDelegationBudgetStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
