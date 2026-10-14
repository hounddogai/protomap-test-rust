use super::*;

use acme_kit::text;

/// Where a budget burn override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnOverrideStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BudgetBurnOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BudgetBurnOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnOverride {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub capacity: i64,
    pub quantity: i64,
    pub rate: f64,
    pub threshold: f64,
    pub active: bool,
    pub owner_id: String,
    pub approved: bool,
    pub port: i64,
    pub status: BudgetBurnOverrideStatus,
}

impl BudgetBurnOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnOverrideStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnOverrideStatus) -> bool {
        match self.status {
            BudgetBurnOverrideStatus::Proposed => {
                match next {
                    BudgetBurnOverrideStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnOverrideStatus::Approved => {
                return next == BudgetBurnOverrideStatus::Rejected || next == BudgetBurnOverrideStatus::Withdrawn;
            }
            BudgetBurnOverrideStatus::Rejected => {
                match next {
                    BudgetBurnOverrideStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BudgetBurnOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.rate + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnOverride {
        BudgetBurnOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            capacity: self.capacity,
            quantity: self.quantity,
            rate: self.rate,
            threshold: self.threshold,
            active: self.active,
            owner_id: self.owner_id.clone(),
            approved: self.approved,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a budget burn override in status proposed.
pub fn new_budget_burn_override(code: &str, title: &str) -> BudgetBurnOverride {
    BudgetBurnOverride {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        capacity: 0,
        quantity: 0,
        rate: 0.0,
        threshold: 0.0,
        active: false,
        owner_id: "".to_string(),
        approved: false,
        port: 0,
        status: BudgetBurnOverrideStatus::Proposed,
    }
}
