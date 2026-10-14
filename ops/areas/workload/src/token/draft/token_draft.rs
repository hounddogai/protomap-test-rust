use super::*;

use acme_kit::text;

/// Where a token draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenDraftStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TokenDraftStatus {
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

impl std::fmt::Display for TokenDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token draft as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenDraft {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub timezone: String,
    pub rotation_days: i64,
    pub name: String,
    pub tags: Vec<String>,
    pub audience: String,
    pub cost: f64,
    pub contact_email: String,
    pub status: TokenDraftStatus,
}

impl TokenDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenDraftStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenDraftStatus) -> bool {
        match self.status {
            TokenDraftStatus::Open => {
                return next == TokenDraftStatus::Resolved || next == TokenDraftStatus::Reopened;
            }
            TokenDraftStatus::InReview => {
                match next {
                    TokenDraftStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenDraftStatus::Resolved => {
                return next == TokenDraftStatus::Archived;
            }
            TokenDraftStatus::Reopened => {
                return next == TokenDraftStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "token draft", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn adjusted_load(&self) -> f64 {
        (self.cost * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenDraft {
        TokenDraft {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            timezone: self.timezone.clone(),
            rotation_days: self.rotation_days,
            name: self.name.clone(),
            tags: self.tags.clone(),
            audience: self.audience.clone(),
            cost: self.cost,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a token draft in status open.
pub fn new_token_draft(code: &str, label: &str) -> TokenDraft {
    TokenDraft {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        timezone: "".to_string(),
        rotation_days: 0,
        name: "".to_string(),
        tags: Vec::new(),
        audience: "".to_string(),
        cost: 0.0,
        contact_email: "".to_string(),
        status: TokenDraftStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_token_draft("code-123", "label-3");
        assert_eq!(item.status, TokenDraftStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_token_draft("code-123", "label-3");
        if item.move_to(TokenDraftStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenDraftStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_token_draft("code-123", "label-3");
        item.status = TokenDraftStatus::Archived;
        assert!(item.move_to(TokenDraftStatus::Open).is_err(), "expected move to fail");
    }
}
