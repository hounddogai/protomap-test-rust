use super::*;

use acme_kit::text;

/// Where a batched evidence mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedEvidenceMappingStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedEvidenceMappingStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedEvidenceMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched evidence mapping as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedEvidenceMapping {
    pub code: String,
    pub title: String,
    pub priority: i64,
    pub rotation_days: i64,
    pub discount: f64,
    pub fingerprint: String,
    pub external_ref: String,
    pub status: BatchedEvidenceMappingStatus,
}

impl BatchedEvidenceMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedEvidenceMappingStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedEvidenceMappingStatus) -> bool {
        match self.status {
            BatchedEvidenceMappingStatus::Proposed => {
                return next == BatchedEvidenceMappingStatus::Approved
                    || next == BatchedEvidenceMappingStatus::Rejected;
            }
            BatchedEvidenceMappingStatus::Approved => {
                match next {
                    BatchedEvidenceMappingStatus::Rejected | BatchedEvidenceMappingStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedEvidenceMappingStatus::Rejected => {
                match next {
                    BatchedEvidenceMappingStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedEvidenceMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched evidence mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.discount + (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedEvidenceMapping {
        BatchedEvidenceMapping {
            code: self.code.clone(),
            title: self.title.clone(),
            priority: self.priority,
            rotation_days: self.rotation_days,
            discount: self.discount,
            fingerprint: self.fingerprint.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a batched evidence mapping in status proposed.
pub fn new_batched_evidence_mapping(code: &str, title: &str) -> BatchedEvidenceMapping {
    BatchedEvidenceMapping {
        code: code.to_string(),
        title: title.to_string(),
        priority: 0,
        rotation_days: 0,
        discount: 0.0,
        fingerprint: "".to_string(),
        external_ref: "".to_string(),
        status: BatchedEvidenceMappingStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_evidence_mapping("code-388", "title-248");
        assert_eq!(item.status, BatchedEvidenceMappingStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_batched_evidence_mapping("code-388", "title-248");
        if item.move_to(BatchedEvidenceMappingStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedEvidenceMappingStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_evidence_mapping("code-388", "title-248");
        item.status = BatchedEvidenceMappingStatus::Withdrawn;
        assert!(
            item.move_to(BatchedEvidenceMappingStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
