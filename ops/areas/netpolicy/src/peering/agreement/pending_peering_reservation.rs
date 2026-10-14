use super::*;

use acme_kit::text;

/// Where a pending peering reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingPeeringReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingPeeringReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingPeeringReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending peering reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPeeringReservation {
    pub code: String,
    pub name: String,
    pub floor_value: f64,
    pub minimum: f64,
    pub attempts: i64,
    pub grace_days: i64,
    pub tenant_id: String,
    pub external_ref: String,
    pub category: String,
    pub notes: String,
    pub status: PendingPeeringReservationStatus,
}

impl PendingPeeringReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingPeeringReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingPeeringReservationStatus) -> bool {
        match self.status {
            PendingPeeringReservationStatus::Created => {
                return next == PendingPeeringReservationStatus::Validated;
            }
            PendingPeeringReservationStatus::Validated => {
                return next == PendingPeeringReservationStatus::Published;
            }
            PendingPeeringReservationStatus::Published => {
                match next {
                    PendingPeeringReservationStatus::Retired => {
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

    pub fn move_to(&mut self, next: PendingPeeringReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending peering reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.attempts as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingPeeringReservation {
        PendingPeeringReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            floor_value: self.floor_value,
            minimum: self.minimum,
            attempts: self.attempts,
            grace_days: self.grace_days,
            tenant_id: self.tenant_id.clone(),
            external_ref: self.external_ref.clone(),
            category: self.category.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a pending peering reservation in status created.
pub fn new_pending_peering_reservation(code: &str, name: &str) -> PendingPeeringReservation {
    PendingPeeringReservation {
        code: code.to_string(),
        name: name.to_string(),
        floor_value: 0.0,
        minimum: 0.0,
        attempts: 0,
        grace_days: 0,
        tenant_id: "".to_string(),
        external_ref: "".to_string(),
        category: "".to_string(),
        notes: "".to_string(),
        status: PendingPeeringReservationStatus::Created,
    }
}
