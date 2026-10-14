use super::*;

use acme_kit::text;

/// Where a trust domain reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainReservationStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TrustDomainReservationStatus {
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

impl std::fmt::Display for TrustDomainReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a trust domain reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainReservation {
    pub code: String,
    pub title: String,
    pub checksum: String,
    pub source: String,
    pub contact_email: String,
    pub ends_at: i64,
    pub active: bool,
    pub status: TrustDomainReservationStatus,
}

impl TrustDomainReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainReservationStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainReservationStatus) -> bool {
        match self.status {
            TrustDomainReservationStatus::Draft => {
                return next == TrustDomainReservationStatus::Closed;
            }
            TrustDomainReservationStatus::Pending => {
                match next {
                    TrustDomainReservationStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainReservationStatus::Active => {
                match next {
                    TrustDomainReservationStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainReservationStatus::Suspended => {
                return next == TrustDomainReservationStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.ends_at as f64
    }

    pub fn copy(&self) -> TrustDomainReservation {
        TrustDomainReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            checksum: self.checksum.clone(),
            source: self.source.clone(),
            contact_email: self.contact_email.clone(),
            ends_at: self.ends_at,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a trust domain reservation in status draft.
pub fn new_trust_domain_reservation(code: &str, title: &str) -> TrustDomainReservation {
    TrustDomainReservation {
        code: code.to_string(),
        title: title.to_string(),
        checksum: "".to_string(),
        source: "".to_string(),
        contact_email: "".to_string(),
        ends_at: 0,
        active: false,
        status: TrustDomainReservationStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_trust_domain_reservation("code-285", "title-343");
        assert_eq!(item.status, TrustDomainReservationStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_trust_domain_reservation("code-285", "title-343");
        if item.move_to(TrustDomainReservationStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainReservationStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_trust_domain_reservation("code-285", "title-343");
        item.status = TrustDomainReservationStatus::Closed;
        assert!(
            item.move_to(TrustDomainReservationStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
