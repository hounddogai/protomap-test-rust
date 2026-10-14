use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a rolling release reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingReleaseReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingReleaseReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for RollingReleaseReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling release reservation as the release team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReleaseReservation {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub revision: i64,
    pub external_ref: String,
    pub amount: f64,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub region: String,
    pub currency: String,
    pub maximum: f64,
    pub status: RollingReleaseReservationStatus,
}

impl RollingReleaseReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingReleaseReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingReleaseReservationStatus) -> bool {
        match self.status {
            RollingReleaseReservationStatus::Open => {
                match next {
                    RollingReleaseReservationStatus::InReview | RollingReleaseReservationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingReleaseReservationStatus::InReview => {
                return next == RollingReleaseReservationStatus::Resolved;
            }
            RollingReleaseReservationStatus::Resolved => {
                return next == RollingReleaseReservationStatus::Archived;
            }
            RollingReleaseReservationStatus::Reopened => {
                match next {
                    RollingReleaseReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingReleaseReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling release reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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

    pub fn weighted_exposure(&self) -> f64 {
        (self.amount * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingReleaseReservation {
        RollingReleaseReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            revision: self.revision,
            external_ref: self.external_ref.clone(),
            amount: self.amount,
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            region: self.region.clone(),
            currency: self.currency.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a rolling release reservation in status open.
pub fn new_rolling_release_reservation(code: &str, name: &str) -> RollingReleaseReservation {
    RollingReleaseReservation {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        revision: 0,
        external_ref: "".to_string(),
        amount: 0.0,
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        region: "".to_string(),
        currency: "".to_string(),
        maximum: 0.0,
        status: RollingReleaseReservationStatus::Open,
    }
}
