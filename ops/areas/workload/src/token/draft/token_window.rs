use super::*;

use acme_kit::text;

/// Where a token window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenWindowStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TokenWindowStatus {
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

impl std::fmt::Display for TokenWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A token window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenWindow {
    pub code: String,
    pub label: String,
    pub locale: String,
    pub ttl_seconds: i64,
    pub minimum: f64,
    pub status: TokenWindowStatus,
}

impl TokenWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenWindowStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenWindowStatus) -> bool {
        match self.status {
            TokenWindowStatus::Draft => {
                match next {
                    TokenWindowStatus::Pending | TokenWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenWindowStatus::Pending => {
                match next {
                    TokenWindowStatus::Suspended | TokenWindowStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenWindowStatus::Active => {
                return next == TokenWindowStatus::Suspended;
            }
            TokenWindowStatus::Suspended => {
                return next == TokenWindowStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.minimum + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenWindow {
        TokenWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            locale: self.locale.clone(),
            ttl_seconds: self.ttl_seconds,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a token window in status draft.
pub fn new_token_window(code: &str, label: &str) -> TokenWindow {
    TokenWindow {
        code: code.to_string(),
        label: label.to_string(),
        locale: "".to_string(),
        ttl_seconds: 0,
        minimum: 0.0,
        status: TokenWindowStatus::Draft,
    }
}
