use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a region pin quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinQuotaStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionPinQuotaStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionPinQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin quota and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinQuota {
    pub code: String,
    pub name: String,
    pub attributes: BTreeMap<String, String>,
    pub capacity: i64,
    pub host: String,
    pub priority: i64,
    pub budget: f64,
    pub active: bool,
    pub reviewer: String,
    pub quantity: i64,
    pub status: RegionPinQuotaStatus,
}

impl RegionPinQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinQuotaStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinQuotaStatus) -> bool {
        match self.status {
            RegionPinQuotaStatus::Requested => {
                return next == RegionPinQuotaStatus::Granted;
            }
            RegionPinQuotaStatus::Granted => {
                match next {
                    RegionPinQuotaStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinQuotaStatus::Revoked => {
                return next == RegionPinQuotaStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "region pin quota", self.code)
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

    pub fn adjusted_value(&self) -> f64 {
        ((self.capacity as f64) * (self.quantity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinQuota {
        RegionPinQuota {
            code: self.code.clone(),
            name: self.name.clone(),
            attributes: self.attributes.clone(),
            capacity: self.capacity,
            host: self.host.clone(),
            priority: self.priority,
            budget: self.budget,
            active: self.active,
            reviewer: self.reviewer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a region pin quota in status requested.
pub fn new_region_pin_quota(code: &str, name: &str) -> RegionPinQuota {
    RegionPinQuota {
        code: code.to_string(),
        name: name.to_string(),
        attributes: BTreeMap::new(),
        capacity: 0,
        host: "".to_string(),
        priority: 0,
        budget: 0.0,
        active: false,
        reviewer: "".to_string(),
        quantity: 0,
        status: RegionPinQuotaStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_region_pin_quota("code-654", "name-688");
        assert_eq!(item.status, RegionPinQuotaStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_region_pin_quota("code-654", "name-688");
        if item.move_to(RegionPinQuotaStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinQuotaStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_region_pin_quota("code-654", "name-688");
        item.status = RegionPinQuotaStatus::Expired;
        assert!(
            item.move_to(RegionPinQuotaStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
