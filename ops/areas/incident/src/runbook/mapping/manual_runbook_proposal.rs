use super::*;

use acme_kit::text;

/// Where a manual runbook proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualRunbookProposalStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ManualRunbookProposalStatus {
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

impl std::fmt::Display for ManualRunbookProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A manual runbook proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualRunbookProposal {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub notes: String,
    pub threshold: f64,
    pub limit: i64,
    pub cipher: String,
    pub status: ManualRunbookProposalStatus,
}

impl ManualRunbookProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualRunbookProposalStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualRunbookProposalStatus) -> bool {
        match self.status {
            ManualRunbookProposalStatus::Draft => {
                return next == ManualRunbookProposalStatus::Active;
            }
            ManualRunbookProposalStatus::Pending => {
                return next == ManualRunbookProposalStatus::Active;
            }
            ManualRunbookProposalStatus::Active => {
                return next == ManualRunbookProposalStatus::Suspended;
            }
            ManualRunbookProposalStatus::Suspended => {
                match next {
                    ManualRunbookProposalStatus::Closed => {
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

    pub fn move_to(&mut self, next: ManualRunbookProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual runbook proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "manual runbook proposal", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.limit as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualRunbookProposal {
        ManualRunbookProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            notes: self.notes.clone(),
            threshold: self.threshold,
            limit: self.limit,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual runbook proposal in status draft.
pub fn new_manual_runbook_proposal(code: &str, title: &str) -> ManualRunbookProposal {
    ManualRunbookProposal {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        notes: "".to_string(),
        threshold: 0.0,
        limit: 0,
        cipher: "".to_string(),
        status: ManualRunbookProposalStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_manual_runbook_proposal("code-444", "title-491");
        assert_eq!(item.status, ManualRunbookProposalStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_manual_runbook_proposal("code-444", "title-491");
        if item.move_to(ManualRunbookProposalStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualRunbookProposalStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_manual_runbook_proposal("code-444", "title-491");
        item.status = ManualRunbookProposalStatus::Closed;
        assert!(
            item.move_to(ManualRunbookProposalStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
