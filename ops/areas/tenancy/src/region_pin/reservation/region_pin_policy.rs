use super::*;

use acme_kit::text;

/// Where a region pin policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinPolicyStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RegionPinPolicyStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RegionPinPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinPolicy {
    pub code: String,
    pub title: String,
    pub weight: f64,
    pub discount: f64,
    pub principal: String,
    pub quantity: i64,
    pub quota_units: i64,
    pub minimum: f64,
    pub ttl_seconds: i64,
    pub status: RegionPinPolicyStatus,
}

impl RegionPinPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinPolicyStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinPolicyStatus) -> bool {
        match self.status {
            RegionPinPolicyStatus::Requested => {
                return next == RegionPinPolicyStatus::Revoked || next == RegionPinPolicyStatus::Expired;
            }
            RegionPinPolicyStatus::Granted => {
                match next {
                    RegionPinPolicyStatus::Revoked | RegionPinPolicyStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionPinPolicyStatus::Revoked => {
                match next {
                    RegionPinPolicyStatus::Expired => {
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

    pub fn move_to(&mut self, next: RegionPinPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin policy {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.quota_units as f64 - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinPolicy {
        RegionPinPolicy {
            code: self.code.clone(),
            title: self.title.clone(),
            weight: self.weight,
            discount: self.discount,
            principal: self.principal.clone(),
            quantity: self.quantity,
            quota_units: self.quota_units,
            minimum: self.minimum,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a region pin policy in status requested.
pub fn new_region_pin_policy(code: &str, title: &str) -> RegionPinPolicy {
    RegionPinPolicy {
        code: code.to_string(),
        title: title.to_string(),
        weight: 0.0,
        discount: 0.0,
        principal: "".to_string(),
        quantity: 0,
        quota_units: 0,
        minimum: 0.0,
        ttl_seconds: 0,
        status: RegionPinPolicyStatus::Requested,
    }
}
