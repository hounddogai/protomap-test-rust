use super::*;

use acme_kit::text;

/// Where a token group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TokenGroupStatus {
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

impl std::fmt::Display for TokenGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token group as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenGroup {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub issuer: String,
    pub amount: f64,
    pub external_ref: String,
    pub tags: Vec<String>,
    pub status: TokenGroupStatus,
}

impl TokenGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenGroupStatus) -> bool {
        match self.status {
            TokenGroupStatus::Open => {
                match next {
                    TokenGroupStatus::InReview | TokenGroupStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenGroupStatus::InReview => {
                match next {
                    TokenGroupStatus::Reopened | TokenGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenGroupStatus::Resolved => {
                match next {
                    TokenGroupStatus::Reopened | TokenGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenGroupStatus::Reopened => {
                match next {
                    TokenGroupStatus::Archived => {
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

    pub fn move_to(&mut self, next: TokenGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TokenGroup {
        TokenGroup {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            issuer: self.issuer.clone(),
            amount: self.amount,
            external_ref: self.external_ref.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a token group in status open.
pub fn new_token_group(code: &str, title: &str) -> TokenGroup {
    TokenGroup {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        issuer: "".to_string(),
        amount: 0.0,
        external_ref: "".to_string(),
        tags: Vec::new(),
        status: TokenGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_token_group("code-359", "title-823");
        assert_eq!(item.status, TokenGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_token_group("code-359", "title-823");
        if item.move_to(TokenGroupStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenGroupStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_token_group("code-359", "title-823");
        item.status = TokenGroupStatus::Archived;
        assert!(item.move_to(TokenGroupStatus::Open).is_err(), "expected move to fail");
    }
}
