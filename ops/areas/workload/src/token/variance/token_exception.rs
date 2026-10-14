use super::*;

use acme_kit::text;

/// Where a token exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenExceptionStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl TokenExceptionStatus {
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

impl std::fmt::Display for TokenExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenException {
    pub code: String,
    pub name: String,
    pub minimum: f64,
    pub tags: Vec<String>,
    pub amount: f64,
    pub visible: bool,
    pub scope: String,
    pub status: TokenExceptionStatus,
}

impl TokenException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenExceptionStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenExceptionStatus) -> bool {
        match self.status {
            TokenExceptionStatus::New => {
                return next == TokenExceptionStatus::Running;
            }
            TokenExceptionStatus::Queued => {
                match next {
                    TokenExceptionStatus::Running | TokenExceptionStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenExceptionStatus::Running => {
                return next == TokenExceptionStatus::Done || next == TokenExceptionStatus::Failed;
            }
            TokenExceptionStatus::Done => {
                match next {
                    TokenExceptionStatus::Failed => {
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

    pub fn move_to(&mut self, next: TokenExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token exception", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        (self.minimum * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenException {
        TokenException {
            code: self.code.clone(),
            name: self.name.clone(),
            minimum: self.minimum,
            tags: self.tags.clone(),
            amount: self.amount,
            visible: self.visible,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a token exception in status new.
pub fn new_token_exception(code: &str, name: &str) -> TokenException {
    TokenException {
        code: code.to_string(),
        name: name.to_string(),
        minimum: 0.0,
        tags: Vec::new(),
        amount: 0.0,
        visible: false,
        scope: "".to_string(),
        status: TokenExceptionStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_token_exception("code-663", "name-125");
        assert_eq!(item.status, TokenExceptionStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_token_exception("code-663", "name-125");
        if item.move_to(TokenExceptionStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenExceptionStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_token_exception("code-663", "name-125");
        item.status = TokenExceptionStatus::Failed;
        assert!(
            item.move_to(TokenExceptionStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
