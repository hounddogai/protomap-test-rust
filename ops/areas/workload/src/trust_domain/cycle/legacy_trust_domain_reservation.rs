use super::*;

use acme_kit::text;

/// Where a legacy trust domain reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyTrustDomainReservationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyTrustDomainReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for LegacyTrustDomainReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy trust domain reservation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyTrustDomainReservation {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub currency: String,
    pub name: String,
    pub created_at: i64,
    pub expires_at: i64,
    pub status: LegacyTrustDomainReservationStatus,
}

impl LegacyTrustDomainReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyTrustDomainReservationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyTrustDomainReservationStatus) -> bool {
        match self.status {
            LegacyTrustDomainReservationStatus::Open => {
                return next == LegacyTrustDomainReservationStatus::InReview
                    || next == LegacyTrustDomainReservationStatus::Reopened;
            }
            LegacyTrustDomainReservationStatus::InReview => {
                return next == LegacyTrustDomainReservationStatus::Resolved
                    || next == LegacyTrustDomainReservationStatus::Archived;
            }
            LegacyTrustDomainReservationStatus::Resolved => {
                return next == LegacyTrustDomainReservationStatus::Reopened
                    || next == LegacyTrustDomainReservationStatus::Archived;
            }
            LegacyTrustDomainReservationStatus::Reopened => {
                match next {
                    LegacyTrustDomainReservationStatus::Archived => {
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

    pub fn move_to(&mut self, next: LegacyTrustDomainReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy trust domain reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy trust domain reservation", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.minimum - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyTrustDomainReservation {
        LegacyTrustDomainReservation {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            currency: self.currency.clone(),
            name: self.name.clone(),
            created_at: self.created_at,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy trust domain reservation in status open.
pub fn new_legacy_trust_domain_reservation(code: &str, title: &str) -> LegacyTrustDomainReservation {
    LegacyTrustDomainReservation {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        currency: "".to_string(),
        name: "".to_string(),
        created_at: 0,
        expires_at: 0,
        status: LegacyTrustDomainReservationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_trust_domain_reservation("code-118", "title-825");
        assert_eq!(item.status, LegacyTrustDomainReservationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_legacy_trust_domain_reservation("code-118", "title-825");
        if item.move_to(LegacyTrustDomainReservationStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyTrustDomainReservationStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_trust_domain_reservation("code-118", "title-825");
        item.status = LegacyTrustDomainReservationStatus::Archived;
        assert!(
            item.move_to(LegacyTrustDomainReservationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
