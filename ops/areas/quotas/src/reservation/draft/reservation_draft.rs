use super::*;

use acme_kit::text;

/// Where a reservation draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationDraftStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ReservationDraftStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ReservationDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation draft as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationDraft {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub namespace: String,
    pub phone: String,
    pub port: i64,
    pub visible: bool,
    pub discount: f64,
    pub summary: String,
    pub principal: String,
    pub threshold: f64,
    pub status: ReservationDraftStatus,
}

impl ReservationDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationDraftStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationDraftStatus) -> bool {
        match self.status {
            ReservationDraftStatus::Proposed => {
                return next == ReservationDraftStatus::Rejected;
            }
            ReservationDraftStatus::Approved => {
                return next == ReservationDraftStatus::Rejected;
            }
            ReservationDraftStatus::Rejected => {
                match next {
                    ReservationDraftStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ReservationDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.port as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationDraft {
        ReservationDraft {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            namespace: self.namespace.clone(),
            phone: self.phone.clone(),
            port: self.port,
            visible: self.visible,
            discount: self.discount,
            summary: self.summary.clone(),
            principal: self.principal.clone(),
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a reservation draft in status proposed.
pub fn new_reservation_draft(code: &str, name: &str) -> ReservationDraft {
    ReservationDraft {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        namespace: "".to_string(),
        phone: "".to_string(),
        port: 0,
        visible: false,
        discount: 0.0,
        summary: "".to_string(),
        principal: "".to_string(),
        threshold: 0.0,
        status: ReservationDraftStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_reservation_draft("code-855", "name-43");
        assert_eq!(item.status, ReservationDraftStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_reservation_draft("code-855", "name-43");
        if item.move_to(ReservationDraftStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReservationDraftStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_reservation_draft("code-855", "name-43");
        item.status = ReservationDraftStatus::Withdrawn;
        assert!(
            item.move_to(ReservationDraftStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
