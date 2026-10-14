use super::*;

use acme_kit::text;

/// Where a identity agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl IdentityAgreementStatus {
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

impl std::fmt::Display for IdentityAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityAgreement {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub margin: f64,
    pub floor_value: f64,
    pub cipher: String,
    pub status: IdentityAgreementStatus,
}

impl IdentityAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityAgreementStatus) -> bool {
        match self.status {
            IdentityAgreementStatus::Open => {
                return next == IdentityAgreementStatus::InReview || next == IdentityAgreementStatus::Reopened;
            }
            IdentityAgreementStatus::InReview => {
                match next {
                    IdentityAgreementStatus::Reopened | IdentityAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityAgreementStatus::Resolved => {
                return next == IdentityAgreementStatus::Archived;
            }
            IdentityAgreementStatus::Reopened => {
                return next == IdentityAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "identity agreement", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.floor_value - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityAgreement {
        IdentityAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            margin: self.margin,
            floor_value: self.floor_value,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a identity agreement in status open.
pub fn new_identity_agreement(code: &str, label: &str) -> IdentityAgreement {
    IdentityAgreement {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        margin: 0.0,
        floor_value: 0.0,
        cipher: "".to_string(),
        status: IdentityAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_identity_agreement("code-293", "label-731");
        assert_eq!(item.status, IdentityAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_identity_agreement("code-293", "label-731");
        if item.move_to(IdentityAgreementStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityAgreementStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_identity_agreement("code-293", "label-731");
        item.status = IdentityAgreementStatus::Archived;
        assert!(
            item.move_to(IdentityAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
