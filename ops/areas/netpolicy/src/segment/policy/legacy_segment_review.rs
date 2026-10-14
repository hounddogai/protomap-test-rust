use super::*;

use acme_kit::text;

/// Where a legacy segment review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacySegmentReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacySegmentReviewStatus {
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

impl std::fmt::Display for LegacySegmentReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy segment review as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacySegmentReview {
    pub code: String,
    pub label: String,
    pub ends_at: i64,
    pub margin: f64,
    pub segment: String,
    pub status: LegacySegmentReviewStatus,
}

impl LegacySegmentReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacySegmentReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacySegmentReviewStatus) -> bool {
        match self.status {
            LegacySegmentReviewStatus::Open => {
                return next == LegacySegmentReviewStatus::Reopened;
            }
            LegacySegmentReviewStatus::InReview => {
                match next {
                    LegacySegmentReviewStatus::Resolved | LegacySegmentReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacySegmentReviewStatus::Resolved => {
                return next == LegacySegmentReviewStatus::Reopened;
            }
            LegacySegmentReviewStatus::Reopened => {
                return next == LegacySegmentReviewStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacySegmentReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy segment review {} cannot move from {} to {}",
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
        format!("{} {}", "legacy segment review", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.margin + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacySegmentReview {
        LegacySegmentReview {
            code: self.code.clone(),
            label: self.label.clone(),
            ends_at: self.ends_at,
            margin: self.margin,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a legacy segment review in status open.
pub fn new_legacy_segment_review(code: &str, label: &str) -> LegacySegmentReview {
    LegacySegmentReview {
        code: code.to_string(),
        label: label.to_string(),
        ends_at: 0,
        margin: 0.0,
        segment: "".to_string(),
        status: LegacySegmentReviewStatus::Open,
    }
}
