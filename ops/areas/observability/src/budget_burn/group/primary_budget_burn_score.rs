use super::*;

use acme_kit::text;

/// Where a primary budget burn score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBudgetBurnScoreStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryBudgetBurnScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryBudgetBurnScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary budget burn score as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBudgetBurnScore {
    pub code: String,
    pub label: String,
    pub floor_value: f64,
    pub contact_email: String,
    pub starts_at: i64,
    pub currency: String,
    pub status: PrimaryBudgetBurnScoreStatus,
}

impl PrimaryBudgetBurnScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBudgetBurnScoreStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBudgetBurnScoreStatus) -> bool {
        match self.status {
            PrimaryBudgetBurnScoreStatus::Proposed => {
                return next == PrimaryBudgetBurnScoreStatus::Approved;
            }
            PrimaryBudgetBurnScoreStatus::Approved => {
                match next {
                    PrimaryBudgetBurnScoreStatus::Rejected | PrimaryBudgetBurnScoreStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBudgetBurnScoreStatus::Rejected => {
                return next == PrimaryBudgetBurnScoreStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryBudgetBurnScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary budget burn score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.floor_value + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryBudgetBurnScore {
        PrimaryBudgetBurnScore {
            code: self.code.clone(),
            label: self.label.clone(),
            floor_value: self.floor_value,
            contact_email: self.contact_email.clone(),
            starts_at: self.starts_at,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary budget burn score in status proposed.
pub fn new_primary_budget_burn_score(code: &str, label: &str) -> PrimaryBudgetBurnScore {
    PrimaryBudgetBurnScore {
        code: code.to_string(),
        label: label.to_string(),
        floor_value: 0.0,
        contact_email: "".to_string(),
        starts_at: 0,
        currency: "".to_string(),
        status: PrimaryBudgetBurnScoreStatus::Proposed,
    }
}
