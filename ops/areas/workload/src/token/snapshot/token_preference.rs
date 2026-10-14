use super::*;

use acme_kit::text;

/// Where a token preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenPreferenceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for TokenPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token preference, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenPreference {
    pub code: String,
    pub title: String,
    pub threshold: f64,
    pub owner_id: String,
    pub limit: i64,
    pub updated_at: i64,
    pub status: TokenPreferenceStatus,
}

impl TokenPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenPreferenceStatus) -> bool {
        match self.status {
            TokenPreferenceStatus::Idle => {
                match next {
                    TokenPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenPreferenceStatus::Warming => {
                match next {
                    TokenPreferenceStatus::Ready | TokenPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenPreferenceStatus::Ready => {
                return next == TokenPreferenceStatus::Stopped;
            }
            TokenPreferenceStatus::Draining => {
                return next == TokenPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.updated_at as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenPreference {
        TokenPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            threshold: self.threshold,
            owner_id: self.owner_id.clone(),
            limit: self.limit,
            updated_at: self.updated_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a token preference in status idle.
pub fn new_token_preference(code: &str, title: &str) -> TokenPreference {
    TokenPreference {
        code: code.to_string(),
        title: title.to_string(),
        threshold: 0.0,
        owner_id: "".to_string(),
        limit: 0,
        updated_at: 0,
        status: TokenPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_token_preference("code-566", "title-973");
        assert_eq!(item.status, TokenPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_token_preference("code-566", "title-973");
        if item.move_to(TokenPreferenceStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenPreferenceStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_token_preference("code-566", "title-973");
        item.status = TokenPreferenceStatus::Stopped;
        assert!(
            item.move_to(TokenPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
