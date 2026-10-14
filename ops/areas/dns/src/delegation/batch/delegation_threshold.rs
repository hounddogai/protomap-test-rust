use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a delegation threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationThresholdStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DelegationThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DelegationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One delegation threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationThreshold {
    pub code: String,
    pub name: String,
    pub cluster: String,
    pub attributes: BTreeMap<String, String>,
    pub category: String,
    pub floor_value: f64,
    pub description: String,
    pub limit: i64,
    pub reviewer: String,
    pub active: bool,
    pub status: DelegationThresholdStatus,
}

impl DelegationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationThresholdStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationThresholdStatus) -> bool {
        match self.status {
            DelegationThresholdStatus::Requested => {
                return next == DelegationThresholdStatus::Revoked;
            }
            DelegationThresholdStatus::Granted => {
                return next == DelegationThresholdStatus::Expired;
            }
            DelegationThresholdStatus::Revoked => {
                match next {
                    DelegationThresholdStatus::Expired => {
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

    pub fn move_to(&mut self, next: DelegationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "delegation threshold", self.code)
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

    pub fn net_value(&self) -> f64 {
        ((self.limit as f64) * self.floor_value * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationThreshold {
        DelegationThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            cluster: self.cluster.clone(),
            attributes: self.attributes.clone(),
            category: self.category.clone(),
            floor_value: self.floor_value,
            description: self.description.clone(),
            limit: self.limit,
            reviewer: self.reviewer.clone(),
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a delegation threshold in status requested.
pub fn new_delegation_threshold(code: &str, name: &str) -> DelegationThreshold {
    DelegationThreshold {
        code: code.to_string(),
        name: name.to_string(),
        cluster: "".to_string(),
        attributes: BTreeMap::new(),
        category: "".to_string(),
        floor_value: 0.0,
        description: "".to_string(),
        limit: 0,
        reviewer: "".to_string(),
        active: false,
        status: DelegationThresholdStatus::Requested,
    }
}
