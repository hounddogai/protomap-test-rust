use super::*;

use acme_kit::text;

/// Where a adaptive zone reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveZoneReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl AdaptiveZoneReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for AdaptiveZoneReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive zone reservation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveZoneReservation {
    pub code: String,
    pub label: String,
    pub fingerprint: String,
    pub source: String,
    pub checksum: String,
    pub approved: bool,
    pub namespace: String,
    pub ttl_seconds: i64,
    pub host: String,
    pub notes: String,
    pub status: AdaptiveZoneReservationStatus,
}

impl AdaptiveZoneReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveZoneReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveZoneReservationStatus) -> bool {
        match self.status {
            AdaptiveZoneReservationStatus::Draft => {
                return next == AdaptiveZoneReservationStatus::Pending;
            }
            AdaptiveZoneReservationStatus::Pending => {
                return next == AdaptiveZoneReservationStatus::Active
                    || next == AdaptiveZoneReservationStatus::Suspended;
            }
            AdaptiveZoneReservationStatus::Active => {
                return next == AdaptiveZoneReservationStatus::Suspended
                    || next == AdaptiveZoneReservationStatus::Closed;
            }
            AdaptiveZoneReservationStatus::Suspended => {
                return next == AdaptiveZoneReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveZoneReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive zone reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> AdaptiveZoneReservation {
        AdaptiveZoneReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            fingerprint: self.fingerprint.clone(),
            source: self.source.clone(),
            checksum: self.checksum.clone(),
            approved: self.approved,
            namespace: self.namespace.clone(),
            ttl_seconds: self.ttl_seconds,
            host: self.host.clone(),
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a adaptive zone reservation in status draft.
pub fn new_adaptive_zone_reservation(code: &str, label: &str) -> AdaptiveZoneReservation {
    AdaptiveZoneReservation {
        code: code.to_string(),
        label: label.to_string(),
        fingerprint: "".to_string(),
        source: "".to_string(),
        checksum: "".to_string(),
        approved: false,
        namespace: "".to_string(),
        ttl_seconds: 0,
        host: "".to_string(),
        notes: "".to_string(),
        status: AdaptiveZoneReservationStatus::Draft,
    }
}
