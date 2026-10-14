use super::*;

use acme_kit::text;

/// Where a regional silence budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalSilenceBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionalSilenceBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionalSilenceBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional silence budget as the silence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalSilenceBudget {
    pub code: String,
    pub label: String,
    pub region: String,
    pub revision: i64,
    pub description: String,
    pub budget: f64,
    pub summary: String,
    pub phone: String,
    pub status: RegionalSilenceBudgetStatus,
}

impl RegionalSilenceBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalSilenceBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalSilenceBudgetStatus) -> bool {
        match self.status {
            RegionalSilenceBudgetStatus::Created => {
                return next == RegionalSilenceBudgetStatus::Validated;
            }
            RegionalSilenceBudgetStatus::Validated => {
                return next == RegionalSilenceBudgetStatus::Published;
            }
            RegionalSilenceBudgetStatus::Published => {
                return next == RegionalSilenceBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalSilenceBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional silence budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional silence budget", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.revision as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalSilenceBudget {
        RegionalSilenceBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            region: self.region.clone(),
            revision: self.revision,
            description: self.description.clone(),
            budget: self.budget,
            summary: self.summary.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a regional silence budget in status created.
pub fn new_regional_silence_budget(code: &str, label: &str) -> RegionalSilenceBudget {
    RegionalSilenceBudget {
        code: code.to_string(),
        label: label.to_string(),
        region: "".to_string(),
        revision: 0,
        description: "".to_string(),
        budget: 0.0,
        summary: "".to_string(),
        phone: "".to_string(),
        status: RegionalSilenceBudgetStatus::Created,
    }
}
