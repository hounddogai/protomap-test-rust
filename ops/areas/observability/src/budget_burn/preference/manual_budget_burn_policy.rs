use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a manual budget burn policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBudgetBurnPolicyStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ManualBudgetBurnPolicyStatus {
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

impl std::fmt::Display for ManualBudgetBurnPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual budget burn policy as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnPolicy {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub capacity: i64,
    pub burst: i64,
    pub channel: String,
    pub status: ManualBudgetBurnPolicyStatus,
}

impl ManualBudgetBurnPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBudgetBurnPolicyStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBudgetBurnPolicyStatus) -> bool {
        match self.status {
            ManualBudgetBurnPolicyStatus::Idle => {
                match next {
                    ManualBudgetBurnPolicyStatus::Ready | ManualBudgetBurnPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBudgetBurnPolicyStatus::Warming => {
                match next {
                    ManualBudgetBurnPolicyStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBudgetBurnPolicyStatus::Ready => {
                match next {
                    ManualBudgetBurnPolicyStatus::Draining | ManualBudgetBurnPolicyStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBudgetBurnPolicyStatus::Draining => {
                return next == ManualBudgetBurnPolicyStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ManualBudgetBurnPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual budget burn policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual budget burn policy", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn projected_total(&self) -> f64 {
        ((self.burst as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualBudgetBurnPolicy {
        ManualBudgetBurnPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            capacity: self.capacity,
            burst: self.burst,
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a manual budget burn policy in status idle.
pub fn new_manual_budget_burn_policy(code: &str, title: &str) -> ManualBudgetBurnPolicy {
    ManualBudgetBurnPolicy {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        capacity: 0,
        burst: 0,
        channel: "".to_string(),
        status: ManualBudgetBurnPolicyStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_manual_budget_burn_policy("code-260", "title-262");
        assert_eq!(item.status, ManualBudgetBurnPolicyStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_manual_budget_burn_policy("code-260", "title-262");
        if item.move_to(ManualBudgetBurnPolicyStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualBudgetBurnPolicyStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_manual_budget_burn_policy("code-260", "title-262");
        item.status = ManualBudgetBurnPolicyStatus::Stopped;
        assert!(
            item.move_to(ManualBudgetBurnPolicyStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
