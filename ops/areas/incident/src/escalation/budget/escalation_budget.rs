use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a escalation budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl EscalationBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for EscalationBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation budget as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationBudget {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub minimum: f64,
    pub audience: String,
    pub attributes: BTreeMap<String, String>,
    pub status: EscalationBudgetStatus,
}

impl EscalationBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationBudgetStatus) -> bool {
        match self.status {
            EscalationBudgetStatus::Created => {
                return next == EscalationBudgetStatus::Retired;
            }
            EscalationBudgetStatus::Validated => {
                return next == EscalationBudgetStatus::Published || next == EscalationBudgetStatus::Retired;
            }
            EscalationBudgetStatus::Published => {
                match next {
                    EscalationBudgetStatus::Retired => {
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

    pub fn move_to(&mut self, next: EscalationBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation budget {} cannot move from {} to {}",
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
        self.minimum
    }

    pub fn copy(&self) -> EscalationBudget {
        EscalationBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            minimum: self.minimum,
            audience: self.audience.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a escalation budget in status created.
pub fn new_escalation_budget(code: &str, title: &str) -> EscalationBudget {
    EscalationBudget {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        minimum: 0.0,
        audience: "".to_string(),
        attributes: BTreeMap::new(),
        status: EscalationBudgetStatus::Created,
    }
}
