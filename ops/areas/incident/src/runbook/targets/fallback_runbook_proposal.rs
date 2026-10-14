use super::*;

use acme_kit::text;

/// Where a fallback runbook proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRunbookProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackRunbookProposalStatus {
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

impl std::fmt::Display for FallbackRunbookProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback runbook proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRunbookProposal {
    pub code: String,
    pub label: String,
    pub checksum: String,
    pub cipher: String,
    pub description: String,
    pub reviewer: String,
    pub country: String,
    pub batch_size: i64,
    pub issuer: String,
    pub locale: String,
    pub rotation_days: i64,
    pub status: FallbackRunbookProposalStatus,
}

impl FallbackRunbookProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRunbookProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRunbookProposalStatus) -> bool {
        match self.status {
            FallbackRunbookProposalStatus::New => {
                match next {
                    FallbackRunbookProposalStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRunbookProposalStatus::Queued => {
                match next {
                    FallbackRunbookProposalStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRunbookProposalStatus::Running => {
                return next == FallbackRunbookProposalStatus::Done || next == FallbackRunbookProposalStatus::Failed;
            }
            FallbackRunbookProposalStatus::Done => {
                match next {
                    FallbackRunbookProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackRunbookProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback runbook proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.batch_size as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRunbookProposal {
        FallbackRunbookProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            checksum: self.checksum.clone(),
            cipher: self.cipher.clone(),
            description: self.description.clone(),
            reviewer: self.reviewer.clone(),
            country: self.country.clone(),
            batch_size: self.batch_size,
            issuer: self.issuer.clone(),
            locale: self.locale.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a fallback runbook proposal in status new.
pub fn new_fallback_runbook_proposal(code: &str, label: &str) -> FallbackRunbookProposal {
    FallbackRunbookProposal {
        code: code.to_string(),
        label: label.to_string(),
        checksum: "".to_string(),
        cipher: "".to_string(),
        description: "".to_string(),
        reviewer: "".to_string(),
        country: "".to_string(),
        batch_size: 0,
        issuer: "".to_string(),
        locale: "".to_string(),
        rotation_days: 0,
        status: FallbackRunbookProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_runbook_proposal("code-492", "label-522");
        assert_eq!(item.status, FallbackRunbookProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_runbook_proposal("code-492", "label-522");
        if item.move_to(FallbackRunbookProposalStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRunbookProposalStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_runbook_proposal("code-492", "label-522");
        item.status = FallbackRunbookProposalStatus::Failed;
        assert!(
            item.move_to(FallbackRunbookProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
