use super::*;

use acme_kit::text;

/// Where a scheduled federation reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledFederationReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ScheduledFederationReservationStatus {
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

impl std::fmt::Display for ScheduledFederationReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled federation reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledFederationReservation {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub key_id: String,
    pub budget: f64,
    pub duration_minutes: i64,
    pub status: ScheduledFederationReservationStatus,
}

impl ScheduledFederationReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledFederationReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledFederationReservationStatus) -> bool {
        match self.status {
            ScheduledFederationReservationStatus::Draft => {
                match next {
                    ScheduledFederationReservationStatus::Pending => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFederationReservationStatus::Pending => {
                match next {
                    ScheduledFederationReservationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFederationReservationStatus::Active => {
                match next {
                    ScheduledFederationReservationStatus::Suspended | ScheduledFederationReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledFederationReservationStatus::Suspended => {
                return next == ScheduledFederationReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledFederationReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled federation reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.budget - (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledFederationReservation {
        ScheduledFederationReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            key_id: self.key_id.clone(),
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a scheduled federation reservation in status draft.
pub fn new_scheduled_federation_reservation(code: &str, title: &str) -> ScheduledFederationReservation {
    ScheduledFederationReservation {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        key_id: "".to_string(),
        budget: 0.0,
        duration_minutes: 0,
        status: ScheduledFederationReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_scheduled_federation_reservation("code-965", "title-794");
        assert_eq!(item.status, ScheduledFederationReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_scheduled_federation_reservation("code-965", "title-794");
        if item.move_to(ScheduledFederationReservationStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledFederationReservationStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_scheduled_federation_reservation("code-965", "title-794");
        item.status = ScheduledFederationReservationStatus::Closed;
        assert!(
            item.move_to(ScheduledFederationReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
