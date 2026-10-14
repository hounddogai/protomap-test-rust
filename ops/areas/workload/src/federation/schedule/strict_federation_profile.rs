use super::*;

use acme_kit::text;

/// Where a strict federation profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFederationProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl StrictFederationProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for StrictFederationProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict federation profile as the federation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFederationProfile {
    pub code: String,
    pub title: String,
    pub amount: f64,
    pub grace_days: i64,
    pub host: String,
    pub visible: bool,
    pub limit: i64,
    pub category: String,
    pub status: StrictFederationProfileStatus,
}

impl StrictFederationProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFederationProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFederationProfileStatus) -> bool {
        match self.status {
            StrictFederationProfileStatus::Proposed => {
                return next == StrictFederationProfileStatus::Approved;
            }
            StrictFederationProfileStatus::Approved => {
                return next == StrictFederationProfileStatus::Rejected;
            }
            StrictFederationProfileStatus::Rejected => {
                return next == StrictFederationProfileStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: StrictFederationProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict federation profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.grace_days as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFederationProfile {
        StrictFederationProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            amount: self.amount,
            grace_days: self.grace_days,
            host: self.host.clone(),
            visible: self.visible,
            limit: self.limit,
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a strict federation profile in status proposed.
pub fn new_strict_federation_profile(code: &str, title: &str) -> StrictFederationProfile {
    StrictFederationProfile {
        code: code.to_string(),
        title: title.to_string(),
        amount: 0.0,
        grace_days: 0,
        host: "".to_string(),
        visible: false,
        limit: 0,
        category: "".to_string(),
        status: StrictFederationProfileStatus::Proposed,
    }
}
