use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a credential cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialCycleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl CredentialCycleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for CredentialCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One credential cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialCycle {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub unit: String,
    pub retries: i64,
    pub status: CredentialCycleStatus,
}

impl CredentialCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialCycleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialCycleStatus) -> bool {
        match self.status {
            CredentialCycleStatus::Proposed => {
                return next == CredentialCycleStatus::Approved || next == CredentialCycleStatus::Withdrawn;
            }
            CredentialCycleStatus::Approved => {
                match next {
                    CredentialCycleStatus::Rejected | CredentialCycleStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialCycleStatus::Rejected => {
                return next == CredentialCycleStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "credential cycle", self.code)
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

    pub fn net_total(&self) -> f64 {
        ((self.retries as f64) * self.weight * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialCycle {
        CredentialCycle {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            weight: self.weight,
            attributes: self.attributes.clone(),
            unit: self.unit.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a credential cycle in status proposed.
pub fn new_credential_cycle(code: &str, name: &str) -> CredentialCycle {
    CredentialCycle {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        weight: 0.0,
        attributes: BTreeMap::new(),
        unit: "".to_string(),
        retries: 0,
        status: CredentialCycleStatus::Proposed,
    }
}
