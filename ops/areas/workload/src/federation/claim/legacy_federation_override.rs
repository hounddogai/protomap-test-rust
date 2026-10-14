use super::*;

use acme_kit::text;

/// Where a legacy federation override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyFederationOverrideStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyFederationOverrideStatus {
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

impl std::fmt::Display for LegacyFederationOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy federation override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyFederationOverride {
    pub code: String,
    pub title: String,
    pub country: String,
    pub retries: i64,
    pub key_id: String,
    pub status: LegacyFederationOverrideStatus,
}

impl LegacyFederationOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyFederationOverrideStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyFederationOverrideStatus) -> bool {
        match self.status {
            LegacyFederationOverrideStatus::Open => {
                return next == LegacyFederationOverrideStatus::Archived;
            }
            LegacyFederationOverrideStatus::InReview => {
                match next {
                    LegacyFederationOverrideStatus::Resolved | LegacyFederationOverrideStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyFederationOverrideStatus::Resolved => {
                return next == LegacyFederationOverrideStatus::Archived;
            }
            LegacyFederationOverrideStatus::Reopened => {
                return next == LegacyFederationOverrideStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyFederationOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy federation override {} cannot move from {} to {}",
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
        self.retries as f64
    }

    pub fn copy(&self) -> LegacyFederationOverride {
        LegacyFederationOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            retries: self.retries,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy federation override in status open.
pub fn new_legacy_federation_override(code: &str, title: &str) -> LegacyFederationOverride {
    LegacyFederationOverride {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        retries: 0,
        key_id: "".to_string(),
        status: LegacyFederationOverrideStatus::Open,
    }
}
