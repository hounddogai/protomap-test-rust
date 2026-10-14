use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a primary binding quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryBindingQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PrimaryBindingQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PrimaryBindingQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary binding quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryBindingQuota {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub quota_units: i64,
    pub description: String,
    pub port: i64,
    pub budget: f64,
    pub attributes: BTreeMap<String, String>,
    pub reviewer: String,
    pub unit: String,
    pub status: PrimaryBindingQuotaStatus,
}

impl PrimaryBindingQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryBindingQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryBindingQuotaStatus) -> bool {
        match self.status {
            PrimaryBindingQuotaStatus::Requested => {
                return next == PrimaryBindingQuotaStatus::Revoked;
            }
            PrimaryBindingQuotaStatus::Granted => {
                match next {
                    PrimaryBindingQuotaStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryBindingQuotaStatus::Revoked => {
                return next == PrimaryBindingQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryBindingQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary binding quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn weighted_value(&self) -> f64 {
        ((self.quota_units as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryBindingQuota {
        PrimaryBindingQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            quota_units: self.quota_units,
            description: self.description.clone(),
            port: self.port,
            budget: self.budget,
            attributes: self.attributes.clone(),
            reviewer: self.reviewer.clone(),
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a primary binding quota in status requested.
pub fn new_primary_binding_quota(code: &str, label: &str) -> PrimaryBindingQuota {
    PrimaryBindingQuota {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        quota_units: 0,
        description: "".to_string(),
        port: 0,
        budget: 0.0,
        attributes: BTreeMap::new(),
        reviewer: "".to_string(),
        unit: "".to_string(),
        status: PrimaryBindingQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_primary_binding_quota("code-411", "label-398");
        assert_eq!(item.status, PrimaryBindingQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_primary_binding_quota("code-411", "label-398");
        if item.move_to(PrimaryBindingQuotaStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryBindingQuotaStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_primary_binding_quota("code-411", "label-398");
        item.status = PrimaryBindingQuotaStatus::Expired;
        assert!(
            item.move_to(PrimaryBindingQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
