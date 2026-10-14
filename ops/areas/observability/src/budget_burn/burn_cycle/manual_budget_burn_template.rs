use super::*;

use acme_kit::text;

/// Where a manual budget burn template is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualBudgetBurnTemplateStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl ManualBudgetBurnTemplateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for ManualBudgetBurnTemplateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual budget burn template, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualBudgetBurnTemplate {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub cluster: String,
    pub archived: bool,
    pub quantity: i64,
    pub status: ManualBudgetBurnTemplateStatus,
}

impl ManualBudgetBurnTemplate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualBudgetBurnTemplateStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualBudgetBurnTemplateStatus) -> bool {
        match self.status {
            ManualBudgetBurnTemplateStatus::Created => {
                match next {
                    ManualBudgetBurnTemplateStatus::Published | ManualBudgetBurnTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBudgetBurnTemplateStatus::Validated => {
                match next {
                    ManualBudgetBurnTemplateStatus::Published | ManualBudgetBurnTemplateStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualBudgetBurnTemplateStatus::Published => {
                match next {
                    ManualBudgetBurnTemplateStatus::Retired => {
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

    pub fn move_to(&mut self, next: ManualBudgetBurnTemplateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual budget burn template {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> ManualBudgetBurnTemplate {
        ManualBudgetBurnTemplate {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            cluster: self.cluster.clone(),
            archived: self.archived,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a manual budget burn template in status created.
pub fn new_manual_budget_burn_template(code: &str, title: &str) -> ManualBudgetBurnTemplate {
    ManualBudgetBurnTemplate {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        cluster: "".to_string(),
        archived: false,
        quantity: 0,
        status: ManualBudgetBurnTemplateStatus::Created,
    }
}
