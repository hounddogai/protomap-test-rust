use super::*;

use acme_kit::text;

/// Where a postmortem budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemBudgetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PostmortemBudgetStatus {
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

impl std::fmt::Display for PostmortemBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem budget. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemBudget {
    pub code: String,
    pub title: String,
    pub unit: String,
    pub namespace: String,
    pub segment: String,
    pub updated_at: i64,
    pub contact_email: String,
    pub owner_id: String,
    pub cipher: String,
    pub duration_minutes: i64,
    pub quota_units: i64,
    pub status: PostmortemBudgetStatus,
}

impl PostmortemBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemBudgetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemBudgetStatus) -> bool {
        match self.status {
            PostmortemBudgetStatus::Open => {
                return next == PostmortemBudgetStatus::InReview;
            }
            PostmortemBudgetStatus::InReview => {
                match next {
                    PostmortemBudgetStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemBudgetStatus::Resolved => {
                return next == PostmortemBudgetStatus::Reopened || next == PostmortemBudgetStatus::Archived;
            }
            PostmortemBudgetStatus::Reopened => {
                return next == PostmortemBudgetStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem budget {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quota_units as f64 + (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemBudget {
        PostmortemBudget {
            code: self.code.clone(),
            title: self.title.clone(),
            unit: self.unit.clone(),
            namespace: self.namespace.clone(),
            segment: self.segment.clone(),
            updated_at: self.updated_at,
            contact_email: self.contact_email.clone(),
            owner_id: self.owner_id.clone(),
            cipher: self.cipher.clone(),
            duration_minutes: self.duration_minutes,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a postmortem budget in status open.
pub fn new_postmortem_budget(code: &str, title: &str) -> PostmortemBudget {
    PostmortemBudget {
        code: code.to_string(),
        title: title.to_string(),
        unit: "".to_string(),
        namespace: "".to_string(),
        segment: "".to_string(),
        updated_at: 0,
        contact_email: "".to_string(),
        owner_id: "".to_string(),
        cipher: "".to_string(),
        duration_minutes: 0,
        quota_units: 0,
        status: PostmortemBudgetStatus::Open,
    }
}
