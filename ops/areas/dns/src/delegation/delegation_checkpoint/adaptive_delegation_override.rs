use super::*;

use acme_kit::text;

/// Where a adaptive delegation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveDelegationOverrideStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveDelegationOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveDelegationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive delegation override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationOverride {
    pub code: String,
    pub name: String,
    pub description: String,
    pub floor_value: f64,
    pub grace_days: i64,
    pub reviewer: String,
    pub ratio: f64,
    pub approved: bool,
    pub channel: String,
    pub cipher: String,
    pub notes: String,
    pub status: AdaptiveDelegationOverrideStatus,
}

impl AdaptiveDelegationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveDelegationOverrideStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveDelegationOverrideStatus) -> bool {
        match self.status {
            AdaptiveDelegationOverrideStatus::Requested => {
                return next == AdaptiveDelegationOverrideStatus::Granted
                    || next == AdaptiveDelegationOverrideStatus::Expired;
            }
            AdaptiveDelegationOverrideStatus::Granted => {
                match next {
                    AdaptiveDelegationOverrideStatus::Revoked | AdaptiveDelegationOverrideStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDelegationOverrideStatus::Revoked => {
                return next == AdaptiveDelegationOverrideStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveDelegationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive delegation override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.ratio + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveDelegationOverride {
        AdaptiveDelegationOverride {
            code: self.code.clone(),
            name: self.name.clone(),
            description: self.description.clone(),
            floor_value: self.floor_value,
            grace_days: self.grace_days,
            reviewer: self.reviewer.clone(),
            ratio: self.ratio,
            approved: self.approved,
            channel: self.channel.clone(),
            cipher: self.cipher.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a adaptive delegation override in status requested.
pub fn new_adaptive_delegation_override(code: &str, name: &str) -> AdaptiveDelegationOverride {
    AdaptiveDelegationOverride {
        code: code.to_string(),
        name: name.to_string(),
        description: "".to_string(),
        floor_value: 0.0,
        grace_days: 0,
        reviewer: "".to_string(),
        ratio: 0.0,
        approved: false,
        channel: "".to_string(),
        cipher: "".to_string(),
        notes: "".to_string(),
        status: AdaptiveDelegationOverrideStatus::Requested,
    }
}
