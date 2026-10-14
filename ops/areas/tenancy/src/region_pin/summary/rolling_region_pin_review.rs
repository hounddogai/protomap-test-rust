use super::*;

use acme_kit::text;

/// Where a rolling region pin review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingRegionPinReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl RollingRegionPinReviewStatus {
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

impl std::fmt::Display for RollingRegionPinReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling region pin review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingRegionPinReview {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub margin: f64,
    pub pinned: bool,
    pub floor_value: f64,
    pub threshold: f64,
    pub created_at: i64,
    pub key_id: String,
    pub status: RollingRegionPinReviewStatus,
}

impl RollingRegionPinReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingRegionPinReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingRegionPinReviewStatus) -> bool {
        match self.status {
            RollingRegionPinReviewStatus::Open => {
                return next == RollingRegionPinReviewStatus::Archived;
            }
            RollingRegionPinReviewStatus::InReview => {
                return next == RollingRegionPinReviewStatus::Resolved
                    || next == RollingRegionPinReviewStatus::Archived;
            }
            RollingRegionPinReviewStatus::Resolved => {
                match next {
                    RollingRegionPinReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingRegionPinReviewStatus::Reopened => {
                match next {
                    RollingRegionPinReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: RollingRegionPinReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling region pin review {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.created_at as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingRegionPinReview {
        RollingRegionPinReview {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            margin: self.margin,
            pinned: self.pinned,
            floor_value: self.floor_value,
            threshold: self.threshold,
            created_at: self.created_at,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a rolling region pin review in status open.
pub fn new_rolling_region_pin_review(code: &str, title: &str) -> RollingRegionPinReview {
    RollingRegionPinReview {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        margin: 0.0,
        pinned: false,
        floor_value: 0.0,
        threshold: 0.0,
        created_at: 0,
        key_id: "".to_string(),
        status: RollingRegionPinReviewStatus::Open,
    }
}
