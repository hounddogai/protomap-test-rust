use super::*;

use acme_kit::text;

/// Where a weighted trust domain agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTrustDomainAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl WeightedTrustDomainAgreementStatus {
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

impl std::fmt::Display for WeightedTrustDomainAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted trust domain agreement as the trust domain team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTrustDomainAgreement {
    pub code: String,
    pub label: String,
    pub issuer: String,
    pub rotation_days: i64,
    pub approved: bool,
    pub sequence: i64,
    pub discount: f64,
    pub status: WeightedTrustDomainAgreementStatus,
}

impl WeightedTrustDomainAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTrustDomainAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTrustDomainAgreementStatus) -> bool {
        match self.status {
            WeightedTrustDomainAgreementStatus::Open => {
                return next == WeightedTrustDomainAgreementStatus::InReview
                    || next == WeightedTrustDomainAgreementStatus::Resolved;
            }
            WeightedTrustDomainAgreementStatus::InReview => {
                match next {
                    WeightedTrustDomainAgreementStatus::Resolved | WeightedTrustDomainAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTrustDomainAgreementStatus::Resolved => {
                return next == WeightedTrustDomainAgreementStatus::Reopened
                    || next == WeightedTrustDomainAgreementStatus::Archived;
            }
            WeightedTrustDomainAgreementStatus::Reopened => {
                match next {
                    WeightedTrustDomainAgreementStatus::Archived => {
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

    pub fn move_to(&mut self, next: WeightedTrustDomainAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted trust domain agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.discount + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTrustDomainAgreement {
        WeightedTrustDomainAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            issuer: self.issuer.clone(),
            rotation_days: self.rotation_days,
            approved: self.approved,
            sequence: self.sequence,
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted trust domain agreement in status open.
pub fn new_weighted_trust_domain_agreement(code: &str, label: &str) -> WeightedTrustDomainAgreement {
    WeightedTrustDomainAgreement {
        code: code.to_string(),
        label: label.to_string(),
        issuer: "".to_string(),
        rotation_days: 0,
        approved: false,
        sequence: 0,
        discount: 0.0,
        status: WeightedTrustDomainAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_weighted_trust_domain_agreement("code-25", "label-494");
        assert_eq!(item.status, WeightedTrustDomainAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_weighted_trust_domain_agreement("code-25", "label-494");
        if item.move_to(WeightedTrustDomainAgreementStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedTrustDomainAgreementStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_weighted_trust_domain_agreement("code-25", "label-494");
        item.status = WeightedTrustDomainAgreementStatus::Archived;
        assert!(
            item.move_to(WeightedTrustDomainAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
