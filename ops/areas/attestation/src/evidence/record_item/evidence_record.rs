use super::*;

use acme_kit::text;

/// Where a evidence record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceRecordStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl EvidenceRecordStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for EvidenceRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence record. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceRecord {
    pub code: String,
    pub label: String,
    pub currency: String,
    pub tenant_id: String,
    pub grace_days: i64,
    pub owner_id: String,
    pub status: EvidenceRecordStatus,
}

impl EvidenceRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceRecordStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceRecordStatus) -> bool {
        match self.status {
            EvidenceRecordStatus::Proposed => {
                match next {
                    EvidenceRecordStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceRecordStatus::Approved => {
                match next {
                    EvidenceRecordStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceRecordStatus::Rejected => {
                return next == EvidenceRecordStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> EvidenceRecord {
        EvidenceRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            currency: self.currency.clone(),
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence record in status proposed.
pub fn new_evidence_record(code: &str, label: &str) -> EvidenceRecord {
    EvidenceRecord {
        code: code.to_string(),
        label: label.to_string(),
        currency: "".to_string(),
        tenant_id: "".to_string(),
        grace_days: 0,
        owner_id: "".to_string(),
        status: EvidenceRecordStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_evidence_record("code-240", "label-702");
        assert_eq!(item.status, EvidenceRecordStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_evidence_record("code-240", "label-702");
        if item.move_to(EvidenceRecordStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceRecordStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_evidence_record("code-240", "label-702");
        item.status = EvidenceRecordStatus::Withdrawn;
        assert!(
            item.move_to(EvidenceRecordStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
