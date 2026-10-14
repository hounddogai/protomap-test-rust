use super::*;

use acme_kit::text;

/// Where a token tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenTierStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TokenTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TokenTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenTier {
    pub code: String,
    pub title: String,
    pub margin: f64,
    pub quantity: i64,
    pub port: i64,
    pub weight: f64,
    pub phone: String,
    pub status: TokenTierStatus,
}

impl TokenTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenTierStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenTierStatus) -> bool {
        match self.status {
            TokenTierStatus::Requested => {
                match next {
                    TokenTierStatus::Granted | TokenTierStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenTierStatus::Granted => {
                return next == TokenTierStatus::Revoked;
            }
            TokenTierStatus::Revoked => {
                match next {
                    TokenTierStatus::Expired => {
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

    pub fn move_to(&mut self, next: TokenTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "token tier", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.port as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenTier {
        TokenTier {
            code: self.code.clone(),
            title: self.title.clone(),
            margin: self.margin,
            quantity: self.quantity,
            port: self.port,
            weight: self.weight,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a token tier in status requested.
pub fn new_token_tier(code: &str, title: &str) -> TokenTier {
    TokenTier {
        code: code.to_string(),
        title: title.to_string(),
        margin: 0.0,
        quantity: 0,
        port: 0,
        weight: 0.0,
        phone: "".to_string(),
        status: TokenTierStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_token_tier("code-452", "title-367");
        assert_eq!(item.status, TokenTierStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_token_tier("code-452", "title-367");
        if item.move_to(TokenTierStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TokenTierStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_token_tier("code-452", "title-367");
        item.status = TokenTierStatus::Expired;
        assert!(
            item.move_to(TokenTierStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
