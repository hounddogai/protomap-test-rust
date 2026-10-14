use super::*;

use acme_kit::text;

/// Where a budget burn threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BudgetBurnThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BudgetBurnThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One budget burn threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnThreshold {
    pub code: String,
    pub title: String,
    pub locale: String,
    pub region: String,
    pub checksum: String,
    pub key_id: String,
    pub quota_units: i64,
    pub active: bool,
    pub threshold: f64,
    pub cluster: String,
    pub status: BudgetBurnThresholdStatus,
}

impl BudgetBurnThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnThresholdStatus) -> bool {
        match self.status {
            BudgetBurnThresholdStatus::Proposed => {
                return next == BudgetBurnThresholdStatus::Rejected;
            }
            BudgetBurnThresholdStatus::Approved => {
                match next {
                    BudgetBurnThresholdStatus::Rejected | BudgetBurnThresholdStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnThresholdStatus::Rejected => {
                match next {
                    BudgetBurnThresholdStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BudgetBurnThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_load(&self) -> f64 {
        ((self.quota_units as f64 - self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnThreshold {
        BudgetBurnThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            locale: self.locale.clone(),
            region: self.region.clone(),
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            quota_units: self.quota_units,
            active: self.active,
            threshold: self.threshold,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a budget burn threshold in status proposed.
pub fn new_budget_burn_threshold(code: &str, title: &str) -> BudgetBurnThreshold {
    BudgetBurnThreshold {
        code: code.to_string(),
        title: title.to_string(),
        locale: "".to_string(),
        region: "".to_string(),
        checksum: "".to_string(),
        key_id: "".to_string(),
        quota_units: 0,
        active: false,
        threshold: 0.0,
        cluster: "".to_string(),
        status: BudgetBurnThresholdStatus::Proposed,
    }
}
