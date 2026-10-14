use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a plan preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanPreferenceStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PlanPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PlanPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a plan preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanPreference {
    pub code: String,
    pub name: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub title: String,
    pub discount: f64,
    pub currency: String,
    pub principal: String,
    pub source: String,
    pub status: PlanPreferenceStatus,
}

impl PlanPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanPreferenceStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanPreferenceStatus) -> bool {
        match self.status {
            PlanPreferenceStatus::Created => {
                return next == PlanPreferenceStatus::Published || next == PlanPreferenceStatus::Retired;
            }
            PlanPreferenceStatus::Validated => {
                match next {
                    PlanPreferenceStatus::Published | PlanPreferenceStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PlanPreferenceStatus::Published => {
                match next {
                    PlanPreferenceStatus::Retired => {
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

    pub fn move_to(&mut self, next: PlanPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
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

    pub fn measured_value(&self) -> f64 {
        self.discount
    }

    pub fn copy(&self) -> PlanPreference {
        PlanPreference {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            title: self.title.clone(),
            discount: self.discount,
            currency: self.currency.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a plan preference in status created.
pub fn new_plan_preference(code: &str, name: &str) -> PlanPreference {
    PlanPreference {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        attributes: BTreeMap::new(),
        title: "".to_string(),
        discount: 0.0,
        currency: "".to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        status: PlanPreferenceStatus::Created,
    }
}
