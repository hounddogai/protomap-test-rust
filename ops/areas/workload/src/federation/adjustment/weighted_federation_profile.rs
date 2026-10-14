use super::*;

use acme_kit::text;

/// Where a weighted federation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFederationProfileStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedFederationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedFederationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted federation profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationProfile {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub floor_value: f64,
    pub quantity: i64,
    pub notes: String,
    pub maximum: f64,
    pub pinned: bool,
    pub threshold: f64,
    pub status: WeightedFederationProfileStatus,
}

impl WeightedFederationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFederationProfileStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFederationProfileStatus) -> bool {
        match self.status {
            WeightedFederationProfileStatus::Requested => {
                match next {
                    WeightedFederationProfileStatus::Granted | WeightedFederationProfileStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationProfileStatus::Granted => {
                return next == WeightedFederationProfileStatus::Revoked
                    || next == WeightedFederationProfileStatus::Expired;
            }
            WeightedFederationProfileStatus::Revoked => {
                return next == WeightedFederationProfileStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedFederationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted federation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.threshold + self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFederationProfile {
        WeightedFederationProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            floor_value: self.floor_value,
            quantity: self.quantity,
            notes: self.notes.clone(),
            maximum: self.maximum,
            pinned: self.pinned,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted federation profile in status requested.
pub fn new_weighted_federation_profile(code: &str, name: &str) -> WeightedFederationProfile {
    WeightedFederationProfile {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        floor_value: 0.0,
        quantity: 0,
        notes: "".to_string(),
        maximum: 0.0,
        pinned: false,
        threshold: 0.0,
        status: WeightedFederationProfileStatus::Requested,
    }
}
