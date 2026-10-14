use super::*;

use acme_kit::text;

/// Where a evidence checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceCheckpointStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for EvidenceCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A evidence checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceCheckpoint {
    pub code: String,
    pub label: String,
    pub grace_days: i64,
    pub checksum: String,
    pub limit: i64,
    pub cipher: String,
    pub version: i64,
    pub status: EvidenceCheckpointStatus,
}

impl EvidenceCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceCheckpointStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceCheckpointStatus) -> bool {
        match self.status {
            EvidenceCheckpointStatus::Draft => {
                return next == EvidenceCheckpointStatus::Active;
            }
            EvidenceCheckpointStatus::Pending => {
                match next {
                    EvidenceCheckpointStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceCheckpointStatus::Active => {
                return next == EvidenceCheckpointStatus::Closed;
            }
            EvidenceCheckpointStatus::Suspended => {
                match next {
                    EvidenceCheckpointStatus::Closed => {
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

    pub fn move_to(&mut self, next: EvidenceCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.limit as f64 - (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceCheckpoint {
        EvidenceCheckpoint {
            code: self.code.clone(),
            label: self.label.clone(),
            grace_days: self.grace_days,
            checksum: self.checksum.clone(),
            limit: self.limit,
            cipher: self.cipher.clone(),
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a evidence checkpoint in status draft.
pub fn new_evidence_checkpoint(code: &str, label: &str) -> EvidenceCheckpoint {
    EvidenceCheckpoint {
        code: code.to_string(),
        label: label.to_string(),
        grace_days: 0,
        checksum: "".to_string(),
        limit: 0,
        cipher: "".to_string(),
        version: 0,
        status: EvidenceCheckpointStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_evidence_checkpoint("code-800", "label-29");
        assert_eq!(item.status, EvidenceCheckpointStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_evidence_checkpoint("code-800", "label-29");
        if item.move_to(EvidenceCheckpointStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EvidenceCheckpointStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_evidence_checkpoint("code-800", "label-29");
        item.status = EvidenceCheckpointStatus::Closed;
        assert!(
            item.move_to(EvidenceCheckpointStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
