use super::*;

use acme_kit::text;

/// Where a adaptive region pin review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveRegionPinReviewStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl AdaptiveRegionPinReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for AdaptiveRegionPinReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive region pin review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveRegionPinReview {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub minimum: f64,
    pub notes: String,
    pub status: AdaptiveRegionPinReviewStatus,
}

impl AdaptiveRegionPinReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveRegionPinReviewStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveRegionPinReviewStatus) -> bool {
        match self.status {
            AdaptiveRegionPinReviewStatus::Created => {
                return next == AdaptiveRegionPinReviewStatus::Published
                    || next == AdaptiveRegionPinReviewStatus::Retired;
            }
            AdaptiveRegionPinReviewStatus::Validated => {
                return next == AdaptiveRegionPinReviewStatus::Retired;
            }
            AdaptiveRegionPinReviewStatus::Published => {
                return next == AdaptiveRegionPinReviewStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveRegionPinReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive region pin review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        (self.discount * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveRegionPinReview {
        AdaptiveRegionPinReview {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            minimum: self.minimum,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a adaptive region pin review in status created.
pub fn new_adaptive_region_pin_review(code: &str, label: &str) -> AdaptiveRegionPinReview {
    AdaptiveRegionPinReview {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        minimum: 0.0,
        notes: "".to_string(),
        status: AdaptiveRegionPinReviewStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_adaptive_region_pin_review("code-296", "label-457");
        assert_eq!(item.status, AdaptiveRegionPinReviewStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_adaptive_region_pin_review("code-296", "label-457");
        if item.move_to(AdaptiveRegionPinReviewStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveRegionPinReviewStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_adaptive_region_pin_review("code-296", "label-457");
        item.status = AdaptiveRegionPinReviewStatus::Retired;
        assert!(
            item.move_to(AdaptiveRegionPinReviewStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
