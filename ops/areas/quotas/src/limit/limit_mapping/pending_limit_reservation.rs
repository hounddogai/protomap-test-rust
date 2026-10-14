use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a pending limit reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingLimitReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PendingLimitReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PendingLimitReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending limit reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingLimitReservation {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub namespace: String,
    pub quota_units: i64,
    pub rate: f64,
    pub attributes: BTreeMap<String, String>,
    pub status: PendingLimitReservationStatus,
}

impl PendingLimitReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingLimitReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingLimitReservationStatus) -> bool {
        match self.status {
            PendingLimitReservationStatus::Requested => {
                return next == PendingLimitReservationStatus::Expired;
            }
            PendingLimitReservationStatus::Granted => {
                return next == PendingLimitReservationStatus::Revoked
                    || next == PendingLimitReservationStatus::Expired;
            }
            PendingLimitReservationStatus::Revoked => {
                return next == PendingLimitReservationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingLimitReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending limit reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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

    pub fn effective_total(&self) -> f64 {
        ((self.quota_units as f64) * self.rate * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingLimitReservation {
        PendingLimitReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            namespace: self.namespace.clone(),
            quota_units: self.quota_units,
            rate: self.rate,
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a pending limit reservation in status requested.
pub fn new_pending_limit_reservation(code: &str, label: &str) -> PendingLimitReservation {
    PendingLimitReservation {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        namespace: "".to_string(),
        quota_units: 0,
        rate: 0.0,
        attributes: BTreeMap::new(),
        status: PendingLimitReservationStatus::Requested,
    }
}
