use super::*;

use acme_kit::text;

/// Where a evidence reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceReservationStatus {
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

impl std::fmt::Display for EvidenceReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence reservation as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceReservation {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub updated_at: i64,
    pub grace_days: i64,
    pub fingerprint: String,
    pub status: EvidenceReservationStatus,
}

impl EvidenceReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceReservationStatus) -> bool {
        match self.status {
            EvidenceReservationStatus::Draft => {
                match next {
                    EvidenceReservationStatus::Pending | EvidenceReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceReservationStatus::Pending => {
                match next {
                    EvidenceReservationStatus::Active | EvidenceReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceReservationStatus::Active => {
                return next == EvidenceReservationStatus::Suspended || next == EvidenceReservationStatus::Closed;
            }
            EvidenceReservationStatus::Suspended => {
                return next == EvidenceReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.updated_at as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceReservation {
        EvidenceReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            updated_at: self.updated_at,
            grace_days: self.grace_days,
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence reservation in status draft.
pub fn new_evidence_reservation(code: &str, label: &str) -> EvidenceReservation {
    EvidenceReservation {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        updated_at: 0,
        grace_days: 0,
        fingerprint: "".to_string(),
        status: EvidenceReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_evidence_reservation("code-293", "label-131");
        assert_eq!(item.status, EvidenceReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_evidence_reservation("code-293", "label-131");
        if item.move_to(EvidenceReservationStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceReservationStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_evidence_reservation("code-293", "label-131");
        item.status = EvidenceReservationStatus::Closed;
        assert!(
            item.move_to(EvidenceReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
