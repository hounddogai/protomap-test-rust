use super::*;

use acme_kit::text;

/// Where a weighted timeline review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedTimelineReviewStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl WeightedTimelineReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for WeightedTimelineReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted timeline review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedTimelineReview {
    pub code: String,
    pub label: String,
    pub visible: bool,
    pub created_at: i64,
    pub quantity: i64,
    pub ceiling: f64,
    pub status: WeightedTimelineReviewStatus,
}

impl WeightedTimelineReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedTimelineReviewStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedTimelineReviewStatus) -> bool {
        match self.status {
            WeightedTimelineReviewStatus::New => {
                return next == WeightedTimelineReviewStatus::Queued;
            }
            WeightedTimelineReviewStatus::Queued => {
                match next {
                    WeightedTimelineReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTimelineReviewStatus::Running => {
                match next {
                    WeightedTimelineReviewStatus::Done | WeightedTimelineReviewStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedTimelineReviewStatus::Done => {
                return next == WeightedTimelineReviewStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedTimelineReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted timeline review {} cannot move from {} to {}",
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
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.created_at as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedTimelineReview {
        WeightedTimelineReview {
            code: self.code.clone(),
            label: self.label.clone(),
            visible: self.visible,
            created_at: self.created_at,
            quantity: self.quantity,
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted timeline review in status new.
pub fn new_weighted_timeline_review(code: &str, label: &str) -> WeightedTimelineReview {
    WeightedTimelineReview {
        code: code.to_string(),
        label: label.to_string(),
        visible: false,
        created_at: 0,
        quantity: 0,
        ceiling: 0.0,
        status: WeightedTimelineReviewStatus::New,
    }
}
