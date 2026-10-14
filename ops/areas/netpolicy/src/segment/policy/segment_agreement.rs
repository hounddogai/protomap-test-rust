use super::*;

use acme_kit::text;

/// Where a segment agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentAgreementStatus {
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

impl std::fmt::Display for SegmentAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment agreement. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAgreement {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub burst: i64,
    pub active: bool,
    pub currency: String,
    pub locale: String,
    pub status: SegmentAgreementStatus,
}

impl SegmentAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentAgreementStatus) -> bool {
        match self.status {
            SegmentAgreementStatus::Open => {
                match next {
                    SegmentAgreementStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentAgreementStatus::InReview => {
                match next {
                    SegmentAgreementStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentAgreementStatus::Resolved => {
                return next == SegmentAgreementStatus::Reopened || next == SegmentAgreementStatus::Archived;
            }
            SegmentAgreementStatus::Reopened => {
                return next == SegmentAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> SegmentAgreement {
        SegmentAgreement {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            burst: self.burst,
            active: self.active,
            currency: self.currency.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a segment agreement in status open.
pub fn new_segment_agreement(code: &str, name: &str) -> SegmentAgreement {
    SegmentAgreement {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        burst: 0,
        active: false,
        currency: "".to_string(),
        locale: "".to_string(),
        status: SegmentAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_segment_agreement("code-650", "name-253");
        assert_eq!(item.status, SegmentAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_segment_agreement("code-650", "name-253");
        if item.move_to(SegmentAgreementStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentAgreementStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_segment_agreement("code-650", "name-253");
        item.status = SegmentAgreementStatus::Archived;
        assert!(
            item.move_to(SegmentAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
