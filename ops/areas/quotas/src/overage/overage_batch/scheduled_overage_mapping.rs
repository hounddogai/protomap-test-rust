use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a scheduled overage mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledOverageMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledOverageMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledOverageMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled overage mapping. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageMapping {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub revision: i64,
    pub weight: f64,
    pub phone: String,
    pub status: ScheduledOverageMappingStatus,
}

impl ScheduledOverageMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledOverageMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledOverageMappingStatus) -> bool {
        match self.status {
            ScheduledOverageMappingStatus::Proposed => {
                match next {
                    ScheduledOverageMappingStatus::Rejected | ScheduledOverageMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledOverageMappingStatus::Approved => {
                match next {
                    ScheduledOverageMappingStatus::Rejected | ScheduledOverageMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledOverageMappingStatus::Rejected => {
                return next == ScheduledOverageMappingStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledOverageMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled overage mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
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

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.weight + (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledOverageMapping {
        ScheduledOverageMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            revision: self.revision,
            weight: self.weight,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled overage mapping in status proposed.
pub fn new_scheduled_overage_mapping(code: &str, title: &str) -> ScheduledOverageMapping {
    ScheduledOverageMapping {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        revision: 0,
        weight: 0.0,
        phone: "".to_string(),
        status: ScheduledOverageMappingStatus::Proposed,
    }
}
