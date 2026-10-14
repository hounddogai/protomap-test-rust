use super::*;

use acme_kit::text;

/// Where a sink review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SinkReviewStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SinkReviewStatus {
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

impl std::fmt::Display for SinkReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A sink review as the sink team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SinkReview {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub fingerprint: String,
    pub tags: Vec<String>,
    pub priority: i64,
    pub notes: String,
    pub audience: String,
    pub status: SinkReviewStatus,
}

impl SinkReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SinkReviewStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SinkReviewStatus) -> bool {
        match self.status {
            SinkReviewStatus::Idle => {
                return next == SinkReviewStatus::Stopped;
            }
            SinkReviewStatus::Warming => {
                return next == SinkReviewStatus::Ready || next == SinkReviewStatus::Draining;
            }
            SinkReviewStatus::Ready => {
                return next == SinkReviewStatus::Draining || next == SinkReviewStatus::Stopped;
            }
            SinkReviewStatus::Draining => {
                return next == SinkReviewStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SinkReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "sink review {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64) * (self.created_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SinkReview {
        SinkReview {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            fingerprint: self.fingerprint.clone(),
            tags: self.tags.clone(),
            priority: self.priority,
            notes: self.notes.clone(),
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a sink review in status idle.
pub fn new_sink_review(code: &str, label: &str) -> SinkReview {
    SinkReview {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        fingerprint: "".to_string(),
        tags: Vec::new(),
        priority: 0,
        notes: "".to_string(),
        audience: "".to_string(),
        status: SinkReviewStatus::Idle,
    }
}
