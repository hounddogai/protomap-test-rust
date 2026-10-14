use super::*;

use acme_kit::text;

/// Where a canary agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CanaryAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CanaryAgreementStatus {
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

impl std::fmt::Display for CanaryAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A canary agreement as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CanaryAgreement {
    pub code: String,
    pub label: String,
    pub notes: String,
    pub margin: f64,
    pub tags: Vec<String>,
    pub quota_units: i64,
    pub currency: String,
    pub status: CanaryAgreementStatus,
}

impl CanaryAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CanaryAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CanaryAgreementStatus) -> bool {
        match self.status {
            CanaryAgreementStatus::Open => {
                match next {
                    CanaryAgreementStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAgreementStatus::InReview => {
                return next == CanaryAgreementStatus::Reopened;
            }
            CanaryAgreementStatus::Resolved => {
                match next {
                    CanaryAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CanaryAgreementStatus::Reopened => {
                return next == CanaryAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CanaryAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "canary agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quota_units as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CanaryAgreement {
        CanaryAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            notes: self.notes.clone(),
            margin: self.margin,
            tags: self.tags.clone(),
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a canary agreement in status open.
pub fn new_canary_agreement(code: &str, label: &str) -> CanaryAgreement {
    CanaryAgreement {
        code: code.to_string(),
        label: label.to_string(),
        notes: "".to_string(),
        margin: 0.0,
        tags: Vec::new(),
        quota_units: 0,
        currency: "".to_string(),
        status: CanaryAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_canary_agreement("code-398", "label-660");
        assert_eq!(item.status, CanaryAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_canary_agreement("code-398", "label-660");
        if item.move_to(CanaryAgreementStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CanaryAgreementStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_canary_agreement("code-398", "label-660");
        item.status = CanaryAgreementStatus::Archived;
        assert!(
            item.move_to(CanaryAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
