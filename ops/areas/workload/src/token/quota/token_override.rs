use super::*;

use acme_kit::text;

/// Where a token override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenOverrideStatus {
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

impl std::fmt::Display for TokenOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token override as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenOverride {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub tags: Vec<String>,
    pub ratio: f64,
    pub phone: String,
    pub status: TokenOverrideStatus,
}

impl TokenOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenOverrideStatus) -> bool {
        match self.status {
            TokenOverrideStatus::Idle => {
                match next {
                    TokenOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenOverrideStatus::Warming => {
                return next == TokenOverrideStatus::Stopped;
            }
            TokenOverrideStatus::Ready => {
                return next == TokenOverrideStatus::Stopped;
            }
            TokenOverrideStatus::Draining => {
                return next == TokenOverrideStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.starts_at as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenOverride {
        TokenOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            tags: self.tags.clone(),
            ratio: self.ratio,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a token override in status idle.
pub fn new_token_override(code: &str, title: &str) -> TokenOverride {
    TokenOverride {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        tags: Vec::new(),
        ratio: 0.0,
        phone: "".to_string(),
        status: TokenOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_token_override("code-257", "title-652");
        assert_eq!(item.status, TokenOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_token_override("code-257", "title-652");
        if item.move_to(TokenOverrideStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenOverrideStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_token_override("code-257", "title-652");
        item.status = TokenOverrideStatus::Stopped;
        assert!(
            item.move_to(TokenOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
