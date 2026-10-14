use super::*;

use acme_kit::text;

/// Where a usage profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl UsageProfileStatus {
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

impl std::fmt::Display for UsageProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a usage profile and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageProfile {
    pub code: String,
    pub label: String,
    pub archived: bool,
    pub updated_at: i64,
    pub ratio: f64,
    pub reviewer: String,
    pub tenant_id: String,
    pub audience: String,
    pub status: UsageProfileStatus,
}

impl UsageProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageProfileStatus) -> bool {
        match self.status {
            UsageProfileStatus::Open => {
                return next == UsageProfileStatus::Archived;
            }
            UsageProfileStatus::InReview => {
                match next {
                    UsageProfileStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageProfileStatus::Resolved => {
                return next == UsageProfileStatus::Archived;
            }
            UsageProfileStatus::Reopened => {
                match next {
                    UsageProfileStatus::Archived => {
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

    pub fn move_to(&mut self, next: UsageProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.updated_at as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageProfile {
        UsageProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            archived: self.archived,
            updated_at: self.updated_at,
            ratio: self.ratio,
            reviewer: self.reviewer.clone(),
            tenant_id: self.tenant_id.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a usage profile in status open.
pub fn new_usage_profile(code: &str, label: &str) -> UsageProfile {
    UsageProfile {
        code: code.to_string(),
        label: label.to_string(),
        archived: false,
        updated_at: 0,
        ratio: 0.0,
        reviewer: "".to_string(),
        tenant_id: "".to_string(),
        audience: "".to_string(),
        status: UsageProfileStatus::Open,
    }
}
