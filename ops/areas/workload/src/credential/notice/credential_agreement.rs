use super::*;

use acme_kit::text;

/// Where a credential agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CredentialAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CredentialAgreementStatus {
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

impl std::fmt::Display for CredentialAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A credential agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct CredentialAgreement {
    pub code: String,
    pub label: String,
    pub maximum: f64,
    pub grace_days: i64,
    pub capacity: i64,
    pub status: CredentialAgreementStatus,
}

impl CredentialAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CredentialAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CredentialAgreementStatus) -> bool {
        match self.status {
            CredentialAgreementStatus::Open => {
                return next == CredentialAgreementStatus::InReview || next == CredentialAgreementStatus::Reopened;
            }
            CredentialAgreementStatus::InReview => {
                match next {
                    CredentialAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialAgreementStatus::Resolved => {
                match next {
                    CredentialAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CredentialAgreementStatus::Reopened => {
                return next == CredentialAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CredentialAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "credential agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "credential agreement", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.capacity as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CredentialAgreement {
        CredentialAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            maximum: self.maximum,
            grace_days: self.grace_days,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a credential agreement in status open.
pub fn new_credential_agreement(code: &str, label: &str) -> CredentialAgreement {
    CredentialAgreement {
        code: code.to_string(),
        label: label.to_string(),
        maximum: 0.0,
        grace_days: 0,
        capacity: 0,
        status: CredentialAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_credential_agreement("code-277", "label-966");
        assert_eq!(item.status, CredentialAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_credential_agreement("code-277", "label-966");
        if item.move_to(CredentialAgreementStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CredentialAgreementStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_credential_agreement("code-277", "label-966");
        item.status = CredentialAgreementStatus::Archived;
        assert!(
            item.move_to(CredentialAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
