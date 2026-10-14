use super::*;

use acme_kit::text;

/// Where a seasonal rollback review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRollbackReviewStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl SeasonalRollbackReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for SeasonalRollbackReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal rollback review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRollbackReview {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub label: String,
    pub tags: Vec<String>,
    pub principal: String,
    pub ttl_seconds: i64,
    pub expires_at: i64,
    pub score: f64,
    pub status: SeasonalRollbackReviewStatus,
}

impl SeasonalRollbackReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRollbackReviewStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRollbackReviewStatus) -> bool {
        match self.status {
            SeasonalRollbackReviewStatus::Planned => {
                match next {
                    SeasonalRollbackReviewStatus::Scheduled | SeasonalRollbackReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRollbackReviewStatus::Scheduled => {
                match next {
                    SeasonalRollbackReviewStatus::InProgress | SeasonalRollbackReviewStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRollbackReviewStatus::InProgress => {
                match next {
                    SeasonalRollbackReviewStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRollbackReviewStatus::Completed => {
                return next == SeasonalRollbackReviewStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRollbackReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal rollback review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRollbackReview {
        SeasonalRollbackReview {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            label: self.label.clone(),
            tags: self.tags.clone(),
            principal: self.principal.clone(),
            ttl_seconds: self.ttl_seconds,
            expires_at: self.expires_at,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a seasonal rollback review in status planned.
pub fn new_seasonal_rollback_review(code: &str, name: &str) -> SeasonalRollbackReview {
    SeasonalRollbackReview {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        label: "".to_string(),
        tags: Vec::new(),
        principal: "".to_string(),
        ttl_seconds: 0,
        expires_at: 0,
        score: 0.0,
        status: SeasonalRollbackReviewStatus::Planned,
    }
}
