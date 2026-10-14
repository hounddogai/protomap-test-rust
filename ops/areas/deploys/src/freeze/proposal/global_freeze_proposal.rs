use super::*;

use acme_kit::text;

/// Where a global freeze proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalFreezeProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl GlobalFreezeProposalStatus {
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

impl std::fmt::Display for GlobalFreezeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global freeze proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalFreezeProposal {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub grace_days: i64,
    pub audience: String,
    pub approved: bool,
    pub locale: String,
    pub status: GlobalFreezeProposalStatus,
}

impl GlobalFreezeProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalFreezeProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalFreezeProposalStatus) -> bool {
        match self.status {
            GlobalFreezeProposalStatus::Open => {
                return next == GlobalFreezeProposalStatus::Reopened;
            }
            GlobalFreezeProposalStatus::InReview => {
                return next == GlobalFreezeProposalStatus::Resolved || next == GlobalFreezeProposalStatus::Reopened;
            }
            GlobalFreezeProposalStatus::Resolved => {
                match next {
                    GlobalFreezeProposalStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalFreezeProposalStatus::Reopened => {
                match next {
                    GlobalFreezeProposalStatus::Archived => {
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

    pub fn move_to(&mut self, next: GlobalFreezeProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global freeze proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> GlobalFreezeProposal {
        GlobalFreezeProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            grace_days: self.grace_days,
            audience: self.audience.clone(),
            approved: self.approved,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global freeze proposal in status open.
pub fn new_global_freeze_proposal(code: &str, name: &str) -> GlobalFreezeProposal {
    GlobalFreezeProposal {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        grace_days: 0,
        audience: "".to_string(),
        approved: false,
        locale: "".to_string(),
        status: GlobalFreezeProposalStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_global_freeze_proposal("code-39", "name-568");
        assert_eq!(item.status, GlobalFreezeProposalStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_global_freeze_proposal("code-39", "name-568");
        if item.move_to(GlobalFreezeProposalStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalFreezeProposalStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_global_freeze_proposal("code-39", "name-568");
        item.status = GlobalFreezeProposalStatus::Archived;
        assert!(
            item.move_to(GlobalFreezeProposalStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
