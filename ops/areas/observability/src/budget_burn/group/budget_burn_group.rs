use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a budget burn group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnGroupStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl BudgetBurnGroupStatus {
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

impl std::fmt::Display for BudgetBurnGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn group and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnGroup {
    pub code: String,
    pub title: String,
    pub owner_id: String,
    pub fingerprint: String,
    pub summary: String,
    pub attributes: BTreeMap<String, String>,
    pub limit: i64,
    pub status: BudgetBurnGroupStatus,
}

impl BudgetBurnGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnGroupStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnGroupStatus) -> bool {
        match self.status {
            BudgetBurnGroupStatus::Idle => {
                return next == BudgetBurnGroupStatus::Ready || next == BudgetBurnGroupStatus::Draining;
            }
            BudgetBurnGroupStatus::Warming => {
                return next == BudgetBurnGroupStatus::Draining || next == BudgetBurnGroupStatus::Stopped;
            }
            BudgetBurnGroupStatus::Ready => {
                match next {
                    BudgetBurnGroupStatus::Draining | BudgetBurnGroupStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnGroupStatus::Draining => {
                return next == BudgetBurnGroupStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BudgetBurnGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
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

    pub fn measured_value(&self) -> f64 {
        self.limit as f64
    }

    pub fn copy(&self) -> BudgetBurnGroup {
        BudgetBurnGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            owner_id: self.owner_id.clone(),
            fingerprint: self.fingerprint.clone(),
            summary: self.summary.clone(),
            attributes: self.attributes.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a budget burn group in status idle.
pub fn new_budget_burn_group(code: &str, title: &str) -> BudgetBurnGroup {
    BudgetBurnGroup {
        code: code.to_string(),
        title: title.to_string(),
        owner_id: "".to_string(),
        fingerprint: "".to_string(),
        summary: "".to_string(),
        attributes: BTreeMap::new(),
        limit: 0,
        status: BudgetBurnGroupStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_budget_burn_group("code-932", "title-360");
        assert_eq!(item.status, BudgetBurnGroupStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_budget_burn_group("code-932", "title-360");
        if item.move_to(BudgetBurnGroupStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BudgetBurnGroupStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_budget_burn_group("code-932", "title-360");
        item.status = BudgetBurnGroupStatus::Stopped;
        assert!(
            item.move_to(BudgetBurnGroupStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
