use super::*;

use acme_kit::text;

/// Where a token variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenVarianceStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TokenVarianceStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for TokenVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One token variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenVariance {
    pub code: String,
    pub title: String,
    pub principal: String,
    pub source: String,
    pub external_ref: String,
    pub amount: f64,
    pub status: TokenVarianceStatus,
}

impl TokenVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenVarianceStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenVarianceStatus) -> bool {
        match self.status {
            TokenVarianceStatus::Draft => {
                match next {
                    TokenVarianceStatus::Pending | TokenVarianceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenVarianceStatus::Pending => {
                return next == TokenVarianceStatus::Closed;
            }
            TokenVarianceStatus::Active => {
                match next {
                    TokenVarianceStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenVarianceStatus::Suspended => {
                match next {
                    TokenVarianceStatus::Closed => {
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

    pub fn move_to(&mut self, next: TokenVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> TokenVariance {
        TokenVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a token variance in status draft.
pub fn new_token_variance(code: &str, title: &str) -> TokenVariance {
    TokenVariance {
        code: code.to_string(),
        title: title.to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        external_ref: "".to_string(),
        amount: 0.0,
        status: TokenVarianceStatus::Draft,
    }
}
