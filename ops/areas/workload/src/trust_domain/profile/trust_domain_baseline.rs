use super::*;

use acme_kit::text;

/// Where a trust domain baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainBaselineStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TrustDomainBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TrustDomainBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One trust domain baseline, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainBaseline {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub budget: f64,
    pub fingerprint: String,
    pub approved: bool,
    pub minimum: f64,
    pub batch_size: i64,
    pub status: TrustDomainBaselineStatus,
}

impl TrustDomainBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainBaselineStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainBaselineStatus) -> bool {
        match self.status {
            TrustDomainBaselineStatus::Requested => {
                return next == TrustDomainBaselineStatus::Granted || next == TrustDomainBaselineStatus::Revoked;
            }
            TrustDomainBaselineStatus::Granted => {
                match next {
                    TrustDomainBaselineStatus::Revoked | TrustDomainBaselineStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainBaselineStatus::Revoked => {
                match next {
                    TrustDomainBaselineStatus::Expired => {
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

    pub fn move_to(&mut self, next: TrustDomainBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.batch_size as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainBaseline {
        TrustDomainBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            budget: self.budget,
            fingerprint: self.fingerprint.clone(),
            approved: self.approved,
            minimum: self.minimum,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a trust domain baseline in status requested.
pub fn new_trust_domain_baseline(code: &str, title: &str) -> TrustDomainBaseline {
    TrustDomainBaseline {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        budget: 0.0,
        fingerprint: "".to_string(),
        approved: false,
        minimum: 0.0,
        batch_size: 0,
        status: TrustDomainBaselineStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_trust_domain_baseline("code-152", "title-97");
        assert_eq!(item.status, TrustDomainBaselineStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_trust_domain_baseline("code-152", "title-97");
        if item.move_to(TrustDomainBaselineStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TrustDomainBaselineStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_trust_domain_baseline("code-152", "title-97");
        item.status = TrustDomainBaselineStatus::Expired;
        assert!(
            item.move_to(TrustDomainBaselineStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
