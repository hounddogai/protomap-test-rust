use super::*;

use acme_kit::text;

/// Where a token batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenBatchStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TokenBatchStatus {
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

impl std::fmt::Display for TokenBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBatch {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub amount: f64,
    pub owner_id: String,
    pub checksum: String,
    pub external_ref: String,
    pub status: TokenBatchStatus,
}

impl TokenBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenBatchStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenBatchStatus) -> bool {
        match self.status {
            TokenBatchStatus::Open => {
                match next {
                    TokenBatchStatus::InReview | TokenBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenBatchStatus::InReview => {
                return next == TokenBatchStatus::Resolved || next == TokenBatchStatus::Reopened;
            }
            TokenBatchStatus::Resolved => {
                match next {
                    TokenBatchStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenBatchStatus::Reopened => {
                match next {
                    TokenBatchStatus::Archived => {
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

    pub fn move_to(&mut self, next: TokenBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token batch", self.code)
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

    pub fn copy(&self) -> TokenBatch {
        TokenBatch {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            amount: self.amount,
            owner_id: self.owner_id.clone(),
            checksum: self.checksum.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a token batch in status open.
pub fn new_token_batch(code: &str, name: &str) -> TokenBatch {
    TokenBatch {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        amount: 0.0,
        owner_id: "".to_string(),
        checksum: "".to_string(),
        external_ref: "".to_string(),
        status: TokenBatchStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_token_batch("code-787", "name-852");
        assert_eq!(item.status, TokenBatchStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_token_batch("code-787", "name-852");
        if item.move_to(TokenBatchStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenBatchStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_token_batch("code-787", "name-852");
        item.status = TokenBatchStatus::Archived;
        assert!(item.move_to(TokenBatchStatus::Open).is_err(), "expected move to fail");
    }
}
