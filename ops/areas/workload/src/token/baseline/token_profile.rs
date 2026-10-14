use super::*;

use acme_kit::text;

/// Where a token profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenProfileStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TokenProfileStatus {
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

impl std::fmt::Display for TokenProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token profile, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenProfile {
    pub code: String,
    pub title: String,
    pub source: String,
    pub external_ref: String,
    pub grace_days: i64,
    pub timezone: String,
    pub region: String,
    pub status: TokenProfileStatus,
}

impl TokenProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenProfileStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenProfileStatus) -> bool {
        match self.status {
            TokenProfileStatus::Idle => {
                match next {
                    TokenProfileStatus::Warming | TokenProfileStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenProfileStatus::Warming => {
                match next {
                    TokenProfileStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenProfileStatus::Ready => {
                return next == TokenProfileStatus::Stopped;
            }
            TokenProfileStatus::Draining => {
                match next {
                    TokenProfileStatus::Stopped => {
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

    pub fn move_to(&mut self, next: TokenProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token profile", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> TokenProfile {
        TokenProfile {
            code: self.code.clone(),
            title: self.title.clone(),
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            grace_days: self.grace_days,
            timezone: self.timezone.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a token profile in status idle.
pub fn new_token_profile(code: &str, title: &str) -> TokenProfile {
    TokenProfile {
        code: code.to_string(),
        title: title.to_string(),
        source: "".to_string(),
        external_ref: "".to_string(),
        grace_days: 0,
        timezone: "".to_string(),
        region: "".to_string(),
        status: TokenProfileStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_token_profile("code-991", "title-151");
        assert_eq!(item.status, TokenProfileStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_token_profile("code-991", "title-151");
        if item.move_to(TokenProfileStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenProfileStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_token_profile("code-991", "title-151");
        item.status = TokenProfileStatus::Stopped;
        assert!(item.move_to(TokenProfileStatus::Idle).is_err(), "expected move to fail");
    }
}
