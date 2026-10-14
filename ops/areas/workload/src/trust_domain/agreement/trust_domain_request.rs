use super::*;

use acme_kit::text;

/// Where a trust domain request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainRequestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TrustDomainRequestStatus {
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

impl std::fmt::Display for TrustDomainRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain request as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainRequest {
    pub code: String,
    pub title: String,
    pub grace_days: i64,
    pub starts_at: i64,
    pub host: String,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub contact_email: String,
    pub visible: bool,
    pub status: TrustDomainRequestStatus,
}

impl TrustDomainRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainRequestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainRequestStatus) -> bool {
        match self.status {
            TrustDomainRequestStatus::Open => {
                match next {
                    TrustDomainRequestStatus::Reopened | TrustDomainRequestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainRequestStatus::InReview => {
                match next {
                    TrustDomainRequestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainRequestStatus::Resolved => {
                return next == TrustDomainRequestStatus::Reopened || next == TrustDomainRequestStatus::Archived;
            }
            TrustDomainRequestStatus::Reopened => {
                return next == TrustDomainRequestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TrustDomainRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.starts_at as f64) * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainRequest {
        TrustDomainRequest {
            code: self.code.clone(),
            title: self.title.clone(),
            grace_days: self.grace_days,
            starts_at: self.starts_at,
            host: self.host.clone(),
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            contact_email: self.contact_email.clone(),
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain request in status open.
pub fn new_trust_domain_request(code: &str, title: &str) -> TrustDomainRequest {
    TrustDomainRequest {
        code: code.to_string(),
        title: title.to_string(),
        grace_days: 0,
        starts_at: 0,
        host: "".to_string(),
        external_ref: "".to_string(),
        duration_minutes: 0,
        contact_email: "".to_string(),
        visible: false,
        status: TrustDomainRequestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_trust_domain_request("code-134", "title-544");
        assert_eq!(item.status, TrustDomainRequestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_trust_domain_request("code-134", "title-544");
        if item.move_to(TrustDomainRequestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainRequestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_trust_domain_request("code-134", "title-544");
        item.status = TrustDomainRequestStatus::Archived;
        assert!(
            item.move_to(TrustDomainRequestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
