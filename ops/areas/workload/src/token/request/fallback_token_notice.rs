use super::*;

use acme_kit::text;

/// Where a fallback token notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackTokenNoticeStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackTokenNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackTokenNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback token notice as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackTokenNotice {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub scope: String,
    pub description: String,
    pub status: FallbackTokenNoticeStatus,
}

impl FallbackTokenNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackTokenNoticeStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackTokenNoticeStatus) -> bool {
        match self.status {
            FallbackTokenNoticeStatus::Requested => {
                match next {
                    FallbackTokenNoticeStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackTokenNoticeStatus::Granted => {
                return next == FallbackTokenNoticeStatus::Revoked || next == FallbackTokenNoticeStatus::Expired;
            }
            FallbackTokenNoticeStatus::Revoked => {
                return next == FallbackTokenNoticeStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackTokenNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback token notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback token notice", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> FallbackTokenNotice {
        FallbackTokenNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            scope: self.scope.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a fallback token notice in status requested.
pub fn new_fallback_token_notice(code: &str, title: &str) -> FallbackTokenNotice {
    FallbackTokenNotice {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        scope: "".to_string(),
        description: "".to_string(),
        status: FallbackTokenNoticeStatus::Requested,
    }
}
