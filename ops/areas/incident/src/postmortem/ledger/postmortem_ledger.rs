use super::*;

use acme_kit::text;

/// Where a postmortem ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PostmortemLedgerStatus {
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

impl std::fmt::Display for PostmortemLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a postmortem ledger and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemLedger {
    pub code: String,
    pub title: String,
    pub active: bool,
    pub retries: i64,
    pub approved: bool,
    pub pinned: bool,
    pub visible: bool,
    pub status: PostmortemLedgerStatus,
}

impl PostmortemLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemLedgerStatus) -> bool {
        match self.status {
            PostmortemLedgerStatus::Open => {
                match next {
                    PostmortemLedgerStatus::InReview | PostmortemLedgerStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemLedgerStatus::InReview => {
                match next {
                    PostmortemLedgerStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemLedgerStatus::Resolved => {
                match next {
                    PostmortemLedgerStatus::Reopened | PostmortemLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemLedgerStatus::Reopened => {
                return next == PostmortemLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> PostmortemLedger {
        PostmortemLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            active: self.active,
            retries: self.retries,
            approved: self.approved,
            pinned: self.pinned,
            visible: self.visible,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a postmortem ledger in status open.
pub fn new_postmortem_ledger(code: &str, title: &str) -> PostmortemLedger {
    PostmortemLedger {
        code: code.to_string(),
        title: title.to_string(),
        active: false,
        retries: 0,
        approved: false,
        pinned: false,
        visible: false,
        status: PostmortemLedgerStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_postmortem_ledger("code-119", "title-345");
        assert_eq!(item.status, PostmortemLedgerStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_postmortem_ledger("code-119", "title-345");
        if item.move_to(PostmortemLedgerStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemLedgerStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_postmortem_ledger("code-119", "title-345");
        item.status = PostmortemLedgerStatus::Archived;
        assert!(
            item.move_to(PostmortemLedgerStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
