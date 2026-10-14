use super::*;

use acme_kit::text;

/// Where a audit event reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AuditEventReservationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AuditEventReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AuditEventReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a audit event reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AuditEventReservation {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub description: String,
    pub rate: f64,
    pub region: String,
    pub pinned: bool,
    pub approved: bool,
    pub contact_email: String,
    pub margin: f64,
    pub tags: Vec<String>,
    pub status: AuditEventReservationStatus,
}

impl AuditEventReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AuditEventReservationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AuditEventReservationStatus) -> bool {
        match self.status {
            AuditEventReservationStatus::Requested => {
                match next {
                    AuditEventReservationStatus::Granted | AuditEventReservationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AuditEventReservationStatus::Granted => {
                return next == AuditEventReservationStatus::Revoked || next == AuditEventReservationStatus::Expired;
            }
            AuditEventReservationStatus::Revoked => {
                match next {
                    AuditEventReservationStatus::Expired => {
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

    pub fn move_to(&mut self, next: AuditEventReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "audit event reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "audit event reservation", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.retries as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AuditEventReservation {
        AuditEventReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            description: self.description.clone(),
            rate: self.rate,
            region: self.region.clone(),
            pinned: self.pinned,
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            margin: self.margin,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a audit event reservation in status requested.
pub fn new_audit_event_reservation(code: &str, title: &str) -> AuditEventReservation {
    AuditEventReservation {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        description: "".to_string(),
        rate: 0.0,
        region: "".to_string(),
        pinned: false,
        approved: false,
        contact_email: "".to_string(),
        margin: 0.0,
        tags: Vec::new(),
        status: AuditEventReservationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_audit_event_reservation("code-242", "title-799");
        assert_eq!(item.status, AuditEventReservationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_audit_event_reservation("code-242", "title-799");
        if item.move_to(AuditEventReservationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AuditEventReservationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_audit_event_reservation("code-242", "title-799");
        item.status = AuditEventReservationStatus::Expired;
        assert!(
            item.move_to(AuditEventReservationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
