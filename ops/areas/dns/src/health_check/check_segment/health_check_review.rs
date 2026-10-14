use super::*;

use acme_kit::text;

/// Where a health check review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl HealthCheckReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for HealthCheckReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A health check review as the health check team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckReview {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub ceiling: f64,
    pub created_at: i64,
    pub revision: i64,
    pub archived: bool,
    pub fingerprint: String,
    pub pinned: bool,
    pub quota_units: i64,
    pub status: HealthCheckReviewStatus,
}

impl HealthCheckReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckReviewStatus) -> bool {
        match self.status {
            HealthCheckReviewStatus::Created => {
                return next == HealthCheckReviewStatus::Validated || next == HealthCheckReviewStatus::Retired;
            }
            HealthCheckReviewStatus::Validated => {
                return next == HealthCheckReviewStatus::Published || next == HealthCheckReviewStatus::Retired;
            }
            HealthCheckReviewStatus::Published => {
                return next == HealthCheckReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: HealthCheckReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.created_at as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckReview {
        HealthCheckReview {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            ceiling: self.ceiling,
            created_at: self.created_at,
            revision: self.revision,
            archived: self.archived,
            fingerprint: self.fingerprint.clone(),
            pinned: self.pinned,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a health check review in status created.
pub fn new_health_check_review(code: &str, title: &str) -> HealthCheckReview {
    HealthCheckReview {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        ceiling: 0.0,
        created_at: 0,
        revision: 0,
        archived: false,
        fingerprint: "".to_string(),
        pinned: false,
        quota_units: 0,
        status: HealthCheckReviewStatus::Created,
    }
}
