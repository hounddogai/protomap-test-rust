use super::*;

use acme_kit::text;

/// Where a lenient nonce preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientNoncePreferenceStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientNoncePreferenceStatus {
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

impl std::fmt::Display for LenientNoncePreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient nonce preference as the nonce team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientNoncePreference {
    pub code: String,
    pub title: String,
    pub retries: i64,
    pub discount: f64,
    pub source: String,
    pub status: LenientNoncePreferenceStatus,
}

impl LenientNoncePreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientNoncePreferenceStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientNoncePreferenceStatus) -> bool {
        match self.status {
            LenientNoncePreferenceStatus::Open => {
                return next == LenientNoncePreferenceStatus::Resolved
                    || next == LenientNoncePreferenceStatus::Archived;
            }
            LenientNoncePreferenceStatus::InReview => {
                return next == LenientNoncePreferenceStatus::Resolved
                    || next == LenientNoncePreferenceStatus::Archived;
            }
            LenientNoncePreferenceStatus::Resolved => {
                match next {
                    LenientNoncePreferenceStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientNoncePreferenceStatus::Reopened => {
                return next == LenientNoncePreferenceStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LenientNoncePreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient nonce preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.retries as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientNoncePreference {
        LenientNoncePreference {
            code: self.code.clone(),
            title: self.title.clone(),
            retries: self.retries,
            discount: self.discount,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a lenient nonce preference in status open.
pub fn new_lenient_nonce_preference(code: &str, title: &str) -> LenientNoncePreference {
    LenientNoncePreference {
        code: code.to_string(),
        title: title.to_string(),
        retries: 0,
        discount: 0.0,
        source: "".to_string(),
        status: LenientNoncePreferenceStatus::Open,
    }
}
