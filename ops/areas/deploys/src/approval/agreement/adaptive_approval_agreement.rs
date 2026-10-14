use super::*;

use acme_kit::text;

/// Where a adaptive approval agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveApprovalAgreementStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl AdaptiveApprovalAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for AdaptiveApprovalAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive approval agreement as the approval team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveApprovalAgreement {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub scope: String,
    pub ceiling: f64,
    pub status: AdaptiveApprovalAgreementStatus,
}

impl AdaptiveApprovalAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveApprovalAgreementStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveApprovalAgreementStatus) -> bool {
        match self.status {
            AdaptiveApprovalAgreementStatus::Requested => {
                match next {
                    AdaptiveApprovalAgreementStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveApprovalAgreementStatus::Granted => {
                match next {
                    AdaptiveApprovalAgreementStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveApprovalAgreementStatus::Revoked => {
                return next == AdaptiveApprovalAgreementStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveApprovalAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive approval agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> AdaptiveApprovalAgreement {
        AdaptiveApprovalAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive approval agreement in status requested.
pub fn new_adaptive_approval_agreement(code: &str, label: &str) -> AdaptiveApprovalAgreement {
    AdaptiveApprovalAgreement {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        scope: "".to_string(),
        ceiling: 0.0,
        status: AdaptiveApprovalAgreementStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_adaptive_approval_agreement("code-810", "label-252");
        assert_eq!(item.status, AdaptiveApprovalAgreementStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_adaptive_approval_agreement("code-810", "label-252");
        if item.move_to(AdaptiveApprovalAgreementStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveApprovalAgreementStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_adaptive_approval_agreement("code-810", "label-252");
        item.status = AdaptiveApprovalAgreementStatus::Expired;
        assert!(
            item.move_to(AdaptiveApprovalAgreementStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
