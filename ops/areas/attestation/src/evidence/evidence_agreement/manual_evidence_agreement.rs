use super::*;

use acme_kit::text;

/// Where a manual evidence agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualEvidenceAgreementStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ManualEvidenceAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ManualEvidenceAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual evidence agreement, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualEvidenceAgreement {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub phone: String,
    pub sequence: i64,
    pub weight: f64,
    pub pinned: bool,
    pub retries: i64,
    pub namespace: String,
    pub expires_at: i64,
    pub source: String,
    pub status: ManualEvidenceAgreementStatus,
}

impl ManualEvidenceAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualEvidenceAgreementStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualEvidenceAgreementStatus) -> bool {
        match self.status {
            ManualEvidenceAgreementStatus::Proposed => {
                match next {
                    ManualEvidenceAgreementStatus::Approved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEvidenceAgreementStatus::Approved => {
                match next {
                    ManualEvidenceAgreementStatus::Rejected | ManualEvidenceAgreementStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ManualEvidenceAgreementStatus::Rejected => {
                match next {
                    ManualEvidenceAgreementStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ManualEvidenceAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual evidence agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "manual evidence agreement", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualEvidenceAgreement {
        ManualEvidenceAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            phone: self.phone.clone(),
            sequence: self.sequence,
            weight: self.weight,
            pinned: self.pinned,
            retries: self.retries,
            namespace: self.namespace.clone(),
            expires_at: self.expires_at,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a manual evidence agreement in status proposed.
pub fn new_manual_evidence_agreement(code: &str, label: &str) -> ManualEvidenceAgreement {
    ManualEvidenceAgreement {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        phone: "".to_string(),
        sequence: 0,
        weight: 0.0,
        pinned: false,
        retries: 0,
        namespace: "".to_string(),
        expires_at: 0,
        source: "".to_string(),
        status: ManualEvidenceAgreementStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_manual_evidence_agreement("code-270", "label-647");
        assert_eq!(item.status, ManualEvidenceAgreementStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_manual_evidence_agreement("code-270", "label-647");
        if item.move_to(ManualEvidenceAgreementStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualEvidenceAgreementStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_manual_evidence_agreement("code-270", "label-647");
        item.status = ManualEvidenceAgreementStatus::Withdrawn;
        assert!(
            item.move_to(ManualEvidenceAgreementStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
