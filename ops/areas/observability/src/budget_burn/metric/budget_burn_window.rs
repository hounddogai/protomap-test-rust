use super::*;

use acme_kit::text;

/// Where a budget burn window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BudgetBurnWindowStatus {
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

impl std::fmt::Display for BudgetBurnWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnWindow {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub title: String,
    pub ttl_seconds: i64,
    pub principal: String,
    pub timezone: String,
    pub rate: f64,
    pub status: BudgetBurnWindowStatus,
}

impl BudgetBurnWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnWindowStatus) -> bool {
        match self.status {
            BudgetBurnWindowStatus::Idle => {
                match next {
                    BudgetBurnWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnWindowStatus::Warming => {
                return next == BudgetBurnWindowStatus::Ready || next == BudgetBurnWindowStatus::Draining;
            }
            BudgetBurnWindowStatus::Ready => {
                match next {
                    BudgetBurnWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnWindowStatus::Draining => {
                return next == BudgetBurnWindowStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.retries as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnWindow {
        BudgetBurnWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            title: self.title.clone(),
            ttl_seconds: self.ttl_seconds,
            principal: self.principal.clone(),
            timezone: self.timezone.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a budget burn window in status idle.
pub fn new_budget_burn_window(code: &str, label: &str) -> BudgetBurnWindow {
    BudgetBurnWindow {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        title: "".to_string(),
        ttl_seconds: 0,
        principal: "".to_string(),
        timezone: "".to_string(),
        rate: 0.0,
        status: BudgetBurnWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_budget_burn_window("code-622", "label-678");
        assert_eq!(item.status, BudgetBurnWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_budget_burn_window("code-622", "label-678");
        if item.move_to(BudgetBurnWindowStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnWindowStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_budget_burn_window("code-622", "label-678");
        item.status = BudgetBurnWindowStatus::Stopped;
        assert!(
            item.move_to(BudgetBurnWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
