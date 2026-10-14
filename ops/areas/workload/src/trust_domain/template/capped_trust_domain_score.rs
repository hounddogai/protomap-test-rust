use super::*;

use acme_kit::text;

/// Where a capped trust domain score is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedTrustDomainScoreStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl CappedTrustDomainScoreStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for CappedTrustDomainScoreStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped trust domain score as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedTrustDomainScore {
    pub code: String,
    pub label: String,
    pub amount: f64,
    pub score: f64,
    pub active: bool,
    pub status: CappedTrustDomainScoreStatus,
}

impl CappedTrustDomainScore {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedTrustDomainScoreStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedTrustDomainScoreStatus) -> bool {
        match self.status {
            CappedTrustDomainScoreStatus::Requested => {
                return next == CappedTrustDomainScoreStatus::Granted || next == CappedTrustDomainScoreStatus::Revoked;
            }
            CappedTrustDomainScoreStatus::Granted => {
                match next {
                    CappedTrustDomainScoreStatus::Revoked | CappedTrustDomainScoreStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedTrustDomainScoreStatus::Revoked => {
                return next == CappedTrustDomainScoreStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedTrustDomainScoreStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped trust domain score {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "capped trust domain score", self.code)
    }

    pub fn net_total(&self) -> f64 {
        ((self.amount - self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedTrustDomainScore {
        CappedTrustDomainScore {
            code: self.code.clone(),
            label: self.label.clone(),
            amount: self.amount,
            score: self.score,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a capped trust domain score in status requested.
pub fn new_capped_trust_domain_score(code: &str, label: &str) -> CappedTrustDomainScore {
    CappedTrustDomainScore {
        code: code.to_string(),
        label: label.to_string(),
        amount: 0.0,
        score: 0.0,
        active: false,
        status: CappedTrustDomainScoreStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_capped_trust_domain_score("code-520", "label-89");
        assert_eq!(item.status, CappedTrustDomainScoreStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_capped_trust_domain_score("code-520", "label-89");
        if item.move_to(CappedTrustDomainScoreStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedTrustDomainScoreStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_capped_trust_domain_score("code-520", "label-89");
        item.status = CappedTrustDomainScoreStatus::Expired;
        assert!(
            item.move_to(CappedTrustDomainScoreStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
