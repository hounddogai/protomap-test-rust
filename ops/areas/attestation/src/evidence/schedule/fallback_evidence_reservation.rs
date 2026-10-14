use super::*;

use acme_kit::text;

/// Where a fallback evidence reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackEvidenceReservationStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FallbackEvidenceReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FallbackEvidenceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback evidence reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackEvidenceReservation {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub description: String,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub status: FallbackEvidenceReservationStatus,
}

impl FallbackEvidenceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackEvidenceReservationStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackEvidenceReservationStatus) -> bool {
        match self.status {
            FallbackEvidenceReservationStatus::Proposed => {
                return next == FallbackEvidenceReservationStatus::Approved
                    || next == FallbackEvidenceReservationStatus::Withdrawn;
            }
            FallbackEvidenceReservationStatus::Approved => {
                match next {
                    FallbackEvidenceReservationStatus::Rejected | FallbackEvidenceReservationStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackEvidenceReservationStatus::Rejected => {
                return next == FallbackEvidenceReservationStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackEvidenceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback evidence reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "fallback evidence reservation", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.duration_minutes as f64
    }

    pub fn copy(&self) -> FallbackEvidenceReservation {
        FallbackEvidenceReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            description: self.description.clone(),
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a fallback evidence reservation in status proposed.
pub fn new_fallback_evidence_reservation(code: &str, label: &str) -> FallbackEvidenceReservation {
    FallbackEvidenceReservation {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        description: "".to_string(),
        duration_minutes: 0,
        external_ref: "".to_string(),
        status: FallbackEvidenceReservationStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_fallback_evidence_reservation("code-439", "label-185");
        assert_eq!(item.status, FallbackEvidenceReservationStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_fallback_evidence_reservation("code-439", "label-185");
        if item.move_to(FallbackEvidenceReservationStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackEvidenceReservationStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_fallback_evidence_reservation("code-439", "label-185");
        item.status = FallbackEvidenceReservationStatus::Withdrawn;
        assert!(
            item.move_to(FallbackEvidenceReservationStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
