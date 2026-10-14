use super::*;

use acme_kit::text;

/// Where a token baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TokenBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TokenBaselineStatus {
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

impl std::fmt::Display for TokenBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a token baseline and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TokenBaseline {
    pub code: String,
    pub label: String,
    pub starts_at: i64,
    pub budget: f64,
    pub ratio: f64,
    pub country: String,
    pub pinned: bool,
    pub locale: String,
    pub active: bool,
    pub summary: String,
    pub status: TokenBaselineStatus,
}

impl TokenBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TokenBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TokenBaselineStatus) -> bool {
        match self.status {
            TokenBaselineStatus::Open => {
                match next {
                    TokenBaselineStatus::InReview | TokenBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenBaselineStatus::InReview => {
                return next == TokenBaselineStatus::Resolved;
            }
            TokenBaselineStatus::Resolved => {
                match next {
                    TokenBaselineStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TokenBaselineStatus::Reopened => {
                return next == TokenBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TokenBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "token baseline {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ratio - (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TokenBaseline {
        TokenBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            starts_at: self.starts_at,
            budget: self.budget,
            ratio: self.ratio,
            country: self.country.clone(),
            pinned: self.pinned,
            locale: self.locale.clone(),
            active: self.active,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a token baseline in status open.
pub fn new_token_baseline(code: &str, label: &str) -> TokenBaseline {
    TokenBaseline {
        code: code.to_string(),
        label: label.to_string(),
        starts_at: 0,
        budget: 0.0,
        ratio: 0.0,
        country: "".to_string(),
        pinned: false,
        locale: "".to_string(),
        active: false,
        summary: "".to_string(),
        status: TokenBaselineStatus::Open,
    }
}
