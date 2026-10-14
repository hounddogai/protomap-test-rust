use super::*;

use acme_kit::text;

/// Where a primary failover review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryFailoverReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl PrimaryFailoverReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for PrimaryFailoverReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary failover review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryFailoverReview {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub host: String,
    pub limit: i64,
    pub ends_at: i64,
    pub title: String,
    pub ratio: f64,
    pub notes: String,
    pub status: PrimaryFailoverReviewStatus,
}

impl PrimaryFailoverReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryFailoverReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryFailoverReviewStatus) -> bool {
        match self.status {
            PrimaryFailoverReviewStatus::Idle => {
                return next == PrimaryFailoverReviewStatus::Ready;
            }
            PrimaryFailoverReviewStatus::Warming => {
                match next {
                    PrimaryFailoverReviewStatus::Draining | PrimaryFailoverReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverReviewStatus::Ready => {
                match next {
                    PrimaryFailoverReviewStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryFailoverReviewStatus::Draining => {
                return next == PrimaryFailoverReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryFailoverReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary failover review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "primary failover review", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ratio + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryFailoverReview {
        PrimaryFailoverReview {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            host: self.host.clone(),
            limit: self.limit,
            ends_at: self.ends_at,
            title: self.title.clone(),
            ratio: self.ratio,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary failover review in status idle.
pub fn new_primary_failover_review(code: &str, label: &str) -> PrimaryFailoverReview {
    PrimaryFailoverReview {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        host: "".to_string(),
        limit: 0,
        ends_at: 0,
        title: "".to_string(),
        ratio: 0.0,
        notes: "".to_string(),
        status: PrimaryFailoverReviewStatus::Idle,
    }
}
