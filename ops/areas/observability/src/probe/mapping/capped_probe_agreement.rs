use super::*;

use acme_kit::text;

/// Where a capped probe agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedProbeAgreementStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedProbeAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for CappedProbeAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped probe agreement as the probe team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedProbeAgreement {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub external_ref: String,
    pub key_id: String,
    pub created_at: i64,
    pub segment: String,
    pub archived: bool,
    pub status: CappedProbeAgreementStatus,
}

impl CappedProbeAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedProbeAgreementStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedProbeAgreementStatus) -> bool {
        match self.status {
            CappedProbeAgreementStatus::New => {
                return next == CappedProbeAgreementStatus::Queued || next == CappedProbeAgreementStatus::Failed;
            }
            CappedProbeAgreementStatus::Queued => {
                match next {
                    CappedProbeAgreementStatus::Running | CappedProbeAgreementStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedProbeAgreementStatus::Running => {
                return next == CappedProbeAgreementStatus::Done || next == CappedProbeAgreementStatus::Failed;
            }
            CappedProbeAgreementStatus::Done => {
                return next == CappedProbeAgreementStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedProbeAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped probe agreement {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.created_at as f64
    }

    pub fn copy(&self) -> CappedProbeAgreement {
        CappedProbeAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            external_ref: self.external_ref.clone(),
            key_id: self.key_id.clone(),
            created_at: self.created_at,
            segment: self.segment.clone(),
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a capped probe agreement in status new.
pub fn new_capped_probe_agreement(code: &str, label: &str) -> CappedProbeAgreement {
    CappedProbeAgreement {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        external_ref: "".to_string(),
        key_id: "".to_string(),
        created_at: 0,
        segment: "".to_string(),
        archived: false,
        status: CappedProbeAgreementStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_probe_agreement("code-329", "label-414");
        assert_eq!(item.status, CappedProbeAgreementStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_capped_probe_agreement("code-329", "label-414");
        if item.move_to(CappedProbeAgreementStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedProbeAgreementStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_probe_agreement("code-329", "label-414");
        item.status = CappedProbeAgreementStatus::Failed;
        assert!(
            item.move_to(CappedProbeAgreementStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
