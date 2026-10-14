use super::*;

use acme_kit::text;

/// Where a tiered segment review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredSegmentReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl TieredSegmentReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for TieredSegmentReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered segment review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredSegmentReview {
    pub code: String,
    pub label: String,
    pub timezone: String,
    pub margin: f64,
    pub grace_days: i64,
    pub scope: String,
    pub updated_at: i64,
    pub maximum: f64,
    pub quota_units: i64,
    pub reviewer: String,
    pub principal: String,
    pub status: TieredSegmentReviewStatus,
}

impl TieredSegmentReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredSegmentReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredSegmentReviewStatus) -> bool {
        match self.status {
            TieredSegmentReviewStatus::Created => {
                match next {
                    TieredSegmentReviewStatus::Validated | TieredSegmentReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSegmentReviewStatus::Validated => {
                match next {
                    TieredSegmentReviewStatus::Published | TieredSegmentReviewStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredSegmentReviewStatus::Published => {
                return next == TieredSegmentReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredSegmentReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered segment review {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.margin - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredSegmentReview {
        TieredSegmentReview {
            code: self.code.clone(),
            label: self.label.clone(),
            timezone: self.timezone.clone(),
            margin: self.margin,
            grace_days: self.grace_days,
            scope: self.scope.clone(),
            updated_at: self.updated_at,
            maximum: self.maximum,
            quota_units: self.quota_units,
            reviewer: self.reviewer.clone(),
            principal: self.principal.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a tiered segment review in status created.
pub fn new_tiered_segment_review(code: &str, label: &str) -> TieredSegmentReview {
    TieredSegmentReview {
        code: code.to_string(),
        label: label.to_string(),
        timezone: "".to_string(),
        margin: 0.0,
        grace_days: 0,
        scope: "".to_string(),
        updated_at: 0,
        maximum: 0.0,
        quota_units: 0,
        reviewer: "".to_string(),
        principal: "".to_string(),
        status: TieredSegmentReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_tiered_segment_review("code-996", "label-895");
        assert_eq!(item.status, TieredSegmentReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_tiered_segment_review("code-996", "label-895");
        if item.move_to(TieredSegmentReviewStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredSegmentReviewStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_tiered_segment_review("code-996", "label-895");
        item.status = TieredSegmentReviewStatus::Retired;
        assert!(
            item.move_to(TieredSegmentReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
