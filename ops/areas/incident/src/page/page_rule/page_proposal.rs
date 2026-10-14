use super::*;

use acme_kit::text;

/// Where a page proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PageProposalStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PageProposalStatus {
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

impl std::fmt::Display for PageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a page proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PageProposal {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub ttl_seconds: i64,
    pub category: String,
    pub audience: String,
    pub status: PageProposalStatus,
}

impl PageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PageProposalStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PageProposalStatus) -> bool {
        match self.status {
            PageProposalStatus::Open => {
                return next == PageProposalStatus::InReview || next == PageProposalStatus::Resolved;
            }
            PageProposalStatus::InReview => {
                match next {
                    PageProposalStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageProposalStatus::Resolved => {
                match next {
                    PageProposalStatus::Reopened | PageProposalStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PageProposalStatus::Reopened => {
                return next == PageProposalStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "page proposal {} cannot move from {} to {}",
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
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> PageProposal {
        PageProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            ttl_seconds: self.ttl_seconds,
            category: self.category.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a page proposal in status open.
pub fn new_page_proposal(code: &str, name: &str) -> PageProposal {
    PageProposal {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        ttl_seconds: 0,
        category: "".to_string(),
        audience: "".to_string(),
        status: PageProposalStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_page_proposal("code-609", "name-837");
        assert_eq!(item.status, PageProposalStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_page_proposal("code-609", "name-837");
        if item.move_to(PageProposalStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PageProposalStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_page_proposal("code-609", "name-837");
        item.status = PageProposalStatus::Archived;
        assert!(item.move_to(PageProposalStatus::Open).is_err(), "expected move to fail");
    }
}
