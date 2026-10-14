use super::*;

use acme_kit::text;

/// Where a capped page proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedPageProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl CappedPageProposalStatus {
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

impl std::fmt::Display for CappedPageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A capped page proposal as the page team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedPageProposal {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub margin: f64,
    pub scope: String,
    pub ceiling: f64,
    pub timezone: String,
    pub checksum: String,
    pub contact_email: String,
    pub maximum: f64,
    pub status: CappedPageProposalStatus,
}

impl CappedPageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedPageProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedPageProposalStatus) -> bool {
        match self.status {
            CappedPageProposalStatus::New => {
                return next == CappedPageProposalStatus::Queued;
            }
            CappedPageProposalStatus::Queued => {
                return next == CappedPageProposalStatus::Failed;
            }
            CappedPageProposalStatus::Running => {
                match next {
                    CappedPageProposalStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedPageProposalStatus::Done => {
                return next == CappedPageProposalStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: CappedPageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped page proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        (self.maximum * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedPageProposal {
        CappedPageProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            margin: self.margin,
            scope: self.scope.clone(),
            ceiling: self.ceiling,
            timezone: self.timezone.clone(),
            checksum: self.checksum.clone(),
            contact_email: self.contact_email.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a capped page proposal in status new.
pub fn new_capped_page_proposal(code: &str, name: &str) -> CappedPageProposal {
    CappedPageProposal {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        margin: 0.0,
        scope: "".to_string(),
        ceiling: 0.0,
        timezone: "".to_string(),
        checksum: "".to_string(),
        contact_email: "".to_string(),
        maximum: 0.0,
        status: CappedPageProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_capped_page_proposal("code-358", "name-672");
        assert_eq!(item.status, CappedPageProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_capped_page_proposal("code-358", "name-672");
        if item.move_to(CappedPageProposalStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedPageProposalStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_capped_page_proposal("code-358", "name-672");
        item.status = CappedPageProposalStatus::Failed;
        assert!(
            item.move_to(CappedPageProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
