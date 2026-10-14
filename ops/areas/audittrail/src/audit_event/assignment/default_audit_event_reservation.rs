use super::*;

use acme_kit::text;

/// Where a default audit event reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultAuditEventReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultAuditEventReservationStatus {
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

impl std::fmt::Display for DefaultAuditEventReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default audit event reservation as the audit event team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultAuditEventReservation {
    pub code: String,
    pub title: String,
    pub reviewer: String,
    pub region: String,
    pub rate: f64,
    pub score: f64,
    pub minimum: f64,
    pub status: DefaultAuditEventReservationStatus,
}

impl DefaultAuditEventReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultAuditEventReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultAuditEventReservationStatus) -> bool {
        match self.status {
            DefaultAuditEventReservationStatus::Draft => {
                match next {
                    DefaultAuditEventReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAuditEventReservationStatus::Pending => {
                match next {
                    DefaultAuditEventReservationStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultAuditEventReservationStatus::Active => {
                return next == DefaultAuditEventReservationStatus::Suspended
                    || next == DefaultAuditEventReservationStatus::Closed;
            }
            DefaultAuditEventReservationStatus::Suspended => {
                return next == DefaultAuditEventReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultAuditEventReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default audit event reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default audit event reservation", self.code)
    }

    pub fn net_value(&self) -> f64 {
        (self.score * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultAuditEventReservation {
        DefaultAuditEventReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            reviewer: self.reviewer.clone(),
            region: self.region.clone(),
            rate: self.rate,
            score: self.score,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a default audit event reservation in status draft.
pub fn new_default_audit_event_reservation(code: &str, title: &str) -> DefaultAuditEventReservation {
    DefaultAuditEventReservation {
        code: code.to_string(),
        title: title.to_string(),
        reviewer: "".to_string(),
        region: "".to_string(),
        rate: 0.0,
        score: 0.0,
        minimum: 0.0,
        status: DefaultAuditEventReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_audit_event_reservation("code-743", "title-531");
        assert_eq!(item.status, DefaultAuditEventReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_default_audit_event_reservation("code-743", "title-531");
        if item.move_to(DefaultAuditEventReservationStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultAuditEventReservationStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_audit_event_reservation("code-743", "title-531");
        item.status = DefaultAuditEventReservationStatus::Closed;
        assert!(
            item.move_to(DefaultAuditEventReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
