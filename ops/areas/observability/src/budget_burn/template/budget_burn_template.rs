use super::*;

use acme_kit::text;

/// Where a budget burn template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BudgetBurnTemplateStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BudgetBurnTemplateStatus {
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

impl std::fmt::Display for BudgetBurnTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A budget burn template as the budget burn team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BudgetBurnTemplate {
    pub code: String,
    pub label: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub timezone: String,
    pub visible: bool,
    pub cipher: String,
    pub duration_minutes: i64,
    pub scope: String,
    pub cost: f64,
    pub status: BudgetBurnTemplateStatus,
}

impl BudgetBurnTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BudgetBurnTemplateStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BudgetBurnTemplateStatus) -> bool {
        match self.status {
            BudgetBurnTemplateStatus::Draft => {
                return next == BudgetBurnTemplateStatus::Pending || next == BudgetBurnTemplateStatus::Closed;
            }
            BudgetBurnTemplateStatus::Pending => {
                match next {
                    BudgetBurnTemplateStatus::Suspended | BudgetBurnTemplateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnTemplateStatus::Active => {
                match next {
                    BudgetBurnTemplateStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BudgetBurnTemplateStatus::Suspended => {
                match next {
                    BudgetBurnTemplateStatus::Closed => {
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

    pub fn move_to(&mut self, next: BudgetBurnTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "budget burn template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.updated_at as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BudgetBurnTemplate {
        BudgetBurnTemplate {
            code: self.code.clone(),
            label: self.label.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            timezone: self.timezone.clone(),
            visible: self.visible,
            cipher: self.cipher.clone(),
            duration_minutes: self.duration_minutes,
            scope: self.scope.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a budget burn template in status draft.
pub fn new_budget_burn_template(code: &str, label: &str) -> BudgetBurnTemplate {
    BudgetBurnTemplate {
        code: code.to_string(),
        label: label.to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        timezone: "".to_string(),
        visible: false,
        cipher: "".to_string(),
        duration_minutes: 0,
        scope: "".to_string(),
        cost: 0.0,
        status: BudgetBurnTemplateStatus::Draft,
    }
}
