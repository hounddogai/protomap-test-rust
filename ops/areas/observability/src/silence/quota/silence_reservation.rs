use super::*;

use acme_kit::text;

/// Where a silence reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SilenceReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SilenceReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SilenceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a silence reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SilenceReservation {
    pub code: String,
    pub name: String,
    pub issuer: String,
    pub cipher: String,
    pub fingerprint: String,
    pub revision: i64,
    pub status: SilenceReservationStatus,
}

impl SilenceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SilenceReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SilenceReservationStatus) -> bool {
        match self.status {
            SilenceReservationStatus::Proposed => {
                return next == SilenceReservationStatus::Approved || next == SilenceReservationStatus::Withdrawn;
            }
            SilenceReservationStatus::Approved => {
                match next {
                    SilenceReservationStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SilenceReservationStatus::Rejected => {
                return next == SilenceReservationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SilenceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "silence reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "silence reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.revision as f64
    }

    pub fn copy(&self) -> SilenceReservation {
        SilenceReservation {
            code: self.code.clone(),
            name: self.name.clone(),
            issuer: self.issuer.clone(),
            cipher: self.cipher.clone(),
            fingerprint: self.fingerprint.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a silence reservation in status proposed.
pub fn new_silence_reservation(code: &str, name: &str) -> SilenceReservation {
    SilenceReservation {
        code: code.to_string(),
        name: name.to_string(),
        issuer: "".to_string(),
        cipher: "".to_string(),
        fingerprint: "".to_string(),
        revision: 0,
        status: SilenceReservationStatus::Proposed,
    }
}
