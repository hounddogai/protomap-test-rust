use super::*;

use acme_kit::text;

/// Where a regional budget burn template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalBudgetBurnTemplateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalBudgetBurnTemplateStatus {
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

impl std::fmt::Display for RegionalBudgetBurnTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A regional budget burn template. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalBudgetBurnTemplate {
    pub code: String,
    pub label: String,
    pub threshold: f64,
    pub batch_size: i64,
    pub quantity: i64,
    pub status: RegionalBudgetBurnTemplateStatus,
}

impl RegionalBudgetBurnTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalBudgetBurnTemplateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalBudgetBurnTemplateStatus) -> bool {
        match self.status {
            RegionalBudgetBurnTemplateStatus::Idle => {
                return next == RegionalBudgetBurnTemplateStatus::Ready
                    || next == RegionalBudgetBurnTemplateStatus::Draining;
            }
            RegionalBudgetBurnTemplateStatus::Warming => {
                return next == RegionalBudgetBurnTemplateStatus::Ready
                    || next == RegionalBudgetBurnTemplateStatus::Stopped;
            }
            RegionalBudgetBurnTemplateStatus::Ready => {
                return next == RegionalBudgetBurnTemplateStatus::Stopped;
            }
            RegionalBudgetBurnTemplateStatus::Draining => {
                return next == RegionalBudgetBurnTemplateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionalBudgetBurnTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional budget burn template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "regional budget burn template", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.quantity as f64 + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionalBudgetBurnTemplate {
        RegionalBudgetBurnTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            threshold: self.threshold,
            batch_size: self.batch_size,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a regional budget burn template in status idle.
pub fn new_regional_budget_burn_template(code: &str, label: &str) -> RegionalBudgetBurnTemplate {
    RegionalBudgetBurnTemplate {
        code: code.to_string(),
        label: label.to_string(),
        threshold: 0.0,
        batch_size: 0,
        quantity: 0,
        status: RegionalBudgetBurnTemplateStatus::Idle,
    }
}
