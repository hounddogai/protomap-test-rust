use super::*;

use acme_kit::text;

/// Where a evidence agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceAgreementStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EvidenceAgreementStatus {
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

impl std::fmt::Display for EvidenceAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceAgreement {
    pub code: String,
    pub label: String,
    pub rotation_days: i64,
    pub discount: f64,
    pub version: i64,
    pub principal: String,
    pub tenant_id: String,
    pub segment: String,
    pub scope: String,
    pub capacity: i64,
    pub status: EvidenceAgreementStatus,
}

impl EvidenceAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceAgreementStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceAgreementStatus) -> bool {
        match self.status {
            EvidenceAgreementStatus::Open => {
                return next == EvidenceAgreementStatus::Reopened;
            }
            EvidenceAgreementStatus::InReview => {
                return next == EvidenceAgreementStatus::Archived;
            }
            EvidenceAgreementStatus::Resolved => {
                return next == EvidenceAgreementStatus::Reopened || next == EvidenceAgreementStatus::Archived;
            }
            EvidenceAgreementStatus::Reopened => {
                return next == EvidenceAgreementStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rotation_days as f64 - (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceAgreement {
        EvidenceAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            rotation_days: self.rotation_days,
            discount: self.discount,
            version: self.version,
            principal: self.principal.clone(),
            tenant_id: self.tenant_id.clone(),
            segment: self.segment.clone(),
            scope: self.scope.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a evidence agreement in status open.
pub fn new_evidence_agreement(code: &str, label: &str) -> EvidenceAgreement {
    EvidenceAgreement {
        code: code.to_string(),
        label: label.to_string(),
        rotation_days: 0,
        discount: 0.0,
        version: 0,
        principal: "".to_string(),
        tenant_id: "".to_string(),
        segment: "".to_string(),
        scope: "".to_string(),
        capacity: 0,
        status: EvidenceAgreementStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_evidence_agreement("code-984", "label-40");
        assert_eq!(item.status, EvidenceAgreementStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_evidence_agreement("code-984", "label-40");
        if item.move_to(EvidenceAgreementStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceAgreementStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_evidence_agreement("code-984", "label-40");
        item.status = EvidenceAgreementStatus::Archived;
        assert!(
            item.move_to(EvidenceAgreementStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
