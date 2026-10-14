use super::*;

use acme_kit::text;

/// Where a manual runbook reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRunbookReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualRunbookReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualRunbookReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual runbook reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookReservation {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub external_ref: String,
    pub namespace: String,
    pub sequence: i64,
    pub budget: f64,
    pub retries: i64,
    pub source: String,
    pub status: ManualRunbookReservationStatus,
}

impl ManualRunbookReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRunbookReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRunbookReservationStatus) -> bool {
        match self.status {
            ManualRunbookReservationStatus::Requested => {
                match next {
                    ManualRunbookReservationStatus::Revoked | ManualRunbookReservationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRunbookReservationStatus::Granted => {
                match next {
                    ManualRunbookReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualRunbookReservationStatus::Revoked => {
                match next {
                    ManualRunbookReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualRunbookReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual runbook reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.sequence as f64 + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRunbookReservation {
        ManualRunbookReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            external_ref: self.external_ref.clone(),
            namespace: self.namespace.clone(),
            sequence: self.sequence,
            budget: self.budget,
            retries: self.retries,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a manual runbook reservation in status requested.
pub fn new_manual_runbook_reservation(code: &str, label: &str) -> ManualRunbookReservation {
    ManualRunbookReservation {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        external_ref: "".to_string(),
        namespace: "".to_string(),
        sequence: 0,
        budget: 0.0,
        retries: 0,
        source: "".to_string(),
        status: ManualRunbookReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_runbook_reservation("code-100", "label-43");
        assert_eq!(item.status, ManualRunbookReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_manual_runbook_reservation("code-100", "label-43");
        if item.move_to(ManualRunbookReservationStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRunbookReservationStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_runbook_reservation("code-100", "label-43");
        item.status = ManualRunbookReservationStatus::Expired;
        assert!(
            item.move_to(ManualRunbookReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
