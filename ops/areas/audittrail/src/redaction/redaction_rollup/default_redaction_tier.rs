use super::*;

use acme_kit::text;

/// Where a default redaction tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultRedactionTierStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DefaultRedactionTierStatus {
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

impl std::fmt::Display for DefaultRedactionTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default redaction tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultRedactionTier {
    pub code: String,
    pub name: String,
    pub source: String,
    pub duration_minutes: i64,
    pub starts_at: i64,
    pub external_ref: String,
    pub attempts: i64,
    pub status: DefaultRedactionTierStatus,
}

impl DefaultRedactionTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultRedactionTierStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultRedactionTierStatus) -> bool {
        match self.status {
            DefaultRedactionTierStatus::Open => {
                return next == DefaultRedactionTierStatus::InReview;
            }
            DefaultRedactionTierStatus::InReview => {
                match next {
                    DefaultRedactionTierStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultRedactionTierStatus::Resolved => {
                return next == DefaultRedactionTierStatus::Reopened || next == DefaultRedactionTierStatus::Archived;
            }
            DefaultRedactionTierStatus::Reopened => {
                match next {
                    DefaultRedactionTierStatus::Archived => {
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

    pub fn move_to(&mut self, next: DefaultRedactionTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default redaction tier {} cannot move from {} to {}",
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
        format!("{} {}", "default redaction tier", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultRedactionTier {
        DefaultRedactionTier {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            duration_minutes: self.duration_minutes,
            starts_at: self.starts_at,
            external_ref: self.external_ref.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default redaction tier in status open.
pub fn new_default_redaction_tier(code: &str, name: &str) -> DefaultRedactionTier {
    DefaultRedactionTier {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        duration_minutes: 0,
        starts_at: 0,
        external_ref: "".to_string(),
        attempts: 0,
        status: DefaultRedactionTierStatus::Open,
    }
}
