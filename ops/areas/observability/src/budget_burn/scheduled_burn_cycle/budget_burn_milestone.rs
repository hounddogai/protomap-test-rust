use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a budget burn milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnMilestoneStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnMilestoneStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for BudgetBurnMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a budget burn milestone and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnMilestone {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub maximum: f64,
    pub audience: String,
    pub score: f64,
    pub version: i64,
    pub priority: i64,
    pub rotation_days: i64,
    pub rate: f64,
    pub status: BudgetBurnMilestoneStatus,
}

impl BudgetBurnMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnMilestoneStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnMilestoneStatus) -> bool {
        match self.status {
            BudgetBurnMilestoneStatus::Draft => {
                match next {
                    BudgetBurnMilestoneStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnMilestoneStatus::Pending => {
                return next == BudgetBurnMilestoneStatus::Active;
            }
            BudgetBurnMilestoneStatus::Active => {
                return next == BudgetBurnMilestoneStatus::Suspended;
            }
            BudgetBurnMilestoneStatus::Suspended => {
                match next {
                    BudgetBurnMilestoneStatus::Closed => {
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

    pub fn move_to(&mut self, next: BudgetBurnMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
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

    pub fn net_total(&self) -> f64 {
        ((self.rate - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnMilestone {
        BudgetBurnMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            maximum: self.maximum,
            audience: self.audience.clone(),
            score: self.score,
            version: self.version,
            priority: self.priority,
            rotation_days: self.rotation_days,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a budget burn milestone in status draft.
pub fn new_budget_burn_milestone(code: &str, name: &str) -> BudgetBurnMilestone {
    BudgetBurnMilestone {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        maximum: 0.0,
        audience: "".to_string(),
        score: 0.0,
        version: 0,
        priority: 0,
        rotation_days: 0,
        rate: 0.0,
        status: BudgetBurnMilestoneStatus::Draft,
    }
}
