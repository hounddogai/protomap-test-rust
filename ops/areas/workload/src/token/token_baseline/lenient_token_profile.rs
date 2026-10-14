use super::*;

use acme_kit::text;

/// Where a lenient token profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTokenProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LenientTokenProfileStatus {
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

impl std::fmt::Display for LenientTokenProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient token profile as the token team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTokenProfile {
    pub code: String,
    pub name: String,
    pub active: bool,
    pub ttl_seconds: i64,
    pub scope: String,
    pub amount: f64,
    pub version: i64,
    pub budget: f64,
    pub priority: i64,
    pub cost: f64,
    pub status: LenientTokenProfileStatus,
}

impl LenientTokenProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTokenProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTokenProfileStatus) -> bool {
        match self.status {
            LenientTokenProfileStatus::Open => {
                return next == LenientTokenProfileStatus::Resolved || next == LenientTokenProfileStatus::Archived;
            }
            LenientTokenProfileStatus::InReview => {
                match next {
                    LenientTokenProfileStatus::Resolved | LenientTokenProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTokenProfileStatus::Resolved => {
                match next {
                    LenientTokenProfileStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTokenProfileStatus::Reopened => {
                match next {
                    LenientTokenProfileStatus::Archived => {
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

    pub fn move_to(&mut self, next: LenientTokenProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient token profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.version as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTokenProfile {
        LenientTokenProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            active: self.active,
            ttl_seconds: self.ttl_seconds,
            scope: self.scope.clone(),
            amount: self.amount,
            version: self.version,
            budget: self.budget,
            priority: self.priority,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a lenient token profile in status open.
pub fn new_lenient_token_profile(code: &str, name: &str) -> LenientTokenProfile {
    LenientTokenProfile {
        code: code.to_string(),
        name: name.to_string(),
        active: false,
        ttl_seconds: 0,
        scope: "".to_string(),
        amount: 0.0,
        version: 0,
        budget: 0.0,
        priority: 0,
        cost: 0.0,
        status: LenientTokenProfileStatus::Open,
    }
}
