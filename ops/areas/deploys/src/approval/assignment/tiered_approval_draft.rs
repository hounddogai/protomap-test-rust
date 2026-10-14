use super::*;

use acme_kit::text;

/// Where a tiered approval draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredApprovalDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TieredApprovalDraftStatus {
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

impl std::fmt::Display for TieredApprovalDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One tiered approval draft, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredApprovalDraft {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub country: String,
    pub contact_email: String,
    pub owner_id: String,
    pub rate: f64,
    pub status: TieredApprovalDraftStatus,
}

impl TieredApprovalDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredApprovalDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredApprovalDraftStatus) -> bool {
        match self.status {
            TieredApprovalDraftStatus::Open => {
                match next {
                    TieredApprovalDraftStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredApprovalDraftStatus::InReview => {
                return next == TieredApprovalDraftStatus::Reopened;
            }
            TieredApprovalDraftStatus::Resolved => {
                return next == TieredApprovalDraftStatus::Reopened;
            }
            TieredApprovalDraftStatus::Reopened => {
                match next {
                    TieredApprovalDraftStatus::Archived => {
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

    pub fn move_to(&mut self, next: TieredApprovalDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered approval draft {} cannot move from {} to {}",
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
        self.rate
    }

    pub fn copy(&self) -> TieredApprovalDraft {
        TieredApprovalDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            country: self.country.clone(),
            contact_email: self.contact_email.clone(),
            owner_id: self.owner_id.clone(),
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a tiered approval draft in status open.
pub fn new_tiered_approval_draft(code: &str, title: &str) -> TieredApprovalDraft {
    TieredApprovalDraft {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        country: "".to_string(),
        contact_email: "".to_string(),
        owner_id: "".to_string(),
        rate: 0.0,
        status: TieredApprovalDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_tiered_approval_draft("code-987", "title-990");
        assert_eq!(item.status, TieredApprovalDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_tiered_approval_draft("code-987", "title-990");
        if item.move_to(TieredApprovalDraftStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredApprovalDraftStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_tiered_approval_draft("code-987", "title-990");
        item.status = TieredApprovalDraftStatus::Archived;
        assert!(
            item.move_to(TieredApprovalDraftStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
