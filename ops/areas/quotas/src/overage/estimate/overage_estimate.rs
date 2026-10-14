use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a overage estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageEstimateStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl OverageEstimateStatus {
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

impl std::fmt::Display for OverageEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One overage estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageEstimate {
    pub code: String,
    pub title: String,
    pub score: f64,
    pub burst: i64,
    pub visible: bool,
    pub created_at: i64,
    pub country: String,
    pub attributes: BTreeMap<String, String>,
    pub contact_email: String,
    pub expires_at: i64,
    pub status: OverageEstimateStatus,
}

impl OverageEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageEstimateStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageEstimateStatus) -> bool {
        match self.status {
            OverageEstimateStatus::Open => {
                return next == OverageEstimateStatus::InReview || next == OverageEstimateStatus::Reopened;
            }
            OverageEstimateStatus::InReview => {
                return next == OverageEstimateStatus::Reopened || next == OverageEstimateStatus::Archived;
            }
            OverageEstimateStatus::Resolved => {
                return next == OverageEstimateStatus::Reopened || next == OverageEstimateStatus::Archived;
            }
            OverageEstimateStatus::Reopened => {
                return next == OverageEstimateStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: OverageEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_load(&self) -> f64 {
        ((self.burst as f64 + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageEstimate {
        OverageEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            score: self.score,
            burst: self.burst,
            visible: self.visible,
            created_at: self.created_at,
            country: self.country.clone(),
            attributes: self.attributes.clone(),
            contact_email: self.contact_email.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a overage estimate in status open.
pub fn new_overage_estimate(code: &str, title: &str) -> OverageEstimate {
    OverageEstimate {
        code: code.to_string(),
        title: title.to_string(),
        score: 0.0,
        burst: 0,
        visible: false,
        created_at: 0,
        country: "".to_string(),
        attributes: BTreeMap::new(),
        contact_email: "".to_string(),
        expires_at: 0,
        status: OverageEstimateStatus::Open,
    }
}
