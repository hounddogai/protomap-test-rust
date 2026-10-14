use super::*;

use acme_kit::text;

/// Where a rolling budget burn preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingBudgetBurnPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingBudgetBurnPreferenceStatus {
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

impl std::fmt::Display for RollingBudgetBurnPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling budget burn preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingBudgetBurnPreference {
    pub code: String,
    pub name: String,
    pub amount: f64,
    pub issuer: String,
    pub channel: String,
    pub status: RollingBudgetBurnPreferenceStatus,
}

impl RollingBudgetBurnPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingBudgetBurnPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingBudgetBurnPreferenceStatus) -> bool {
        match self.status {
            RollingBudgetBurnPreferenceStatus::Idle => {
                return next == RollingBudgetBurnPreferenceStatus::Warming
                    || next == RollingBudgetBurnPreferenceStatus::Stopped;
            }
            RollingBudgetBurnPreferenceStatus::Warming => {
                match next {
                    RollingBudgetBurnPreferenceStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnPreferenceStatus::Ready => {
                match next {
                    RollingBudgetBurnPreferenceStatus::Draining | RollingBudgetBurnPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingBudgetBurnPreferenceStatus::Draining => {
                match next {
                    RollingBudgetBurnPreferenceStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RollingBudgetBurnPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling budget burn preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> RollingBudgetBurnPreference {
        RollingBudgetBurnPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            amount: self.amount,
            issuer: self.issuer.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a rolling budget burn preference in status idle.
pub fn new_rolling_budget_burn_preference(code: &str, name: &str) -> RollingBudgetBurnPreference {
    RollingBudgetBurnPreference {
        code: code.to_string(),
        name: name.to_string(),
        amount: 0.0,
        issuer: "".to_string(),
        channel: "".to_string(),
        status: RollingBudgetBurnPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_budget_burn_preference("code-363", "name-693");
        assert_eq!(item.status, RollingBudgetBurnPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_rolling_budget_burn_preference("code-363", "name-693");
        if item.move_to(RollingBudgetBurnPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingBudgetBurnPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_budget_burn_preference("code-363", "name-693");
        item.status = RollingBudgetBurnPreferenceStatus::Stopped;
        assert!(
            item.move_to(RollingBudgetBurnPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
