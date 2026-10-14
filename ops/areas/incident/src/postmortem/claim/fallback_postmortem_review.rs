use super::*;

use acme_kit::text;

/// Where a fallback postmortem review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPostmortemReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FallbackPostmortemReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for FallbackPostmortemReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback postmortem review as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPostmortemReview {
    pub code: String,
    pub title: String,
    pub starts_at: i64,
    pub quota_units: i64,
    pub duration_minutes: i64,
    pub quantity: i64,
    pub host: String,
    pub locale: String,
    pub status: FallbackPostmortemReviewStatus,
}

impl FallbackPostmortemReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPostmortemReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPostmortemReviewStatus) -> bool {
        match self.status {
            FallbackPostmortemReviewStatus::Draft => {
                return next == FallbackPostmortemReviewStatus::Suspended;
            }
            FallbackPostmortemReviewStatus::Pending => {
                return next == FallbackPostmortemReviewStatus::Active
                    || next == FallbackPostmortemReviewStatus::Suspended;
            }
            FallbackPostmortemReviewStatus::Active => {
                match next {
                    FallbackPostmortemReviewStatus::Suspended | FallbackPostmortemReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPostmortemReviewStatus::Suspended => {
                return next == FallbackPostmortemReviewStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackPostmortemReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback postmortem review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.quantity as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPostmortemReview {
        FallbackPostmortemReview {
            code: self.code.clone(),
            title: self.title.clone(),
            starts_at: self.starts_at,
            quota_units: self.quota_units,
            duration_minutes: self.duration_minutes,
            quantity: self.quantity,
            host: self.host.clone(),
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback postmortem review in status draft.
pub fn new_fallback_postmortem_review(code: &str, title: &str) -> FallbackPostmortemReview {
    FallbackPostmortemReview {
        code: code.to_string(),
        title: title.to_string(),
        starts_at: 0,
        quota_units: 0,
        duration_minutes: 0,
        quantity: 0,
        host: "".to_string(),
        locale: "".to_string(),
        status: FallbackPostmortemReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_fallback_postmortem_review("code-656", "title-492");
        assert_eq!(item.status, FallbackPostmortemReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_fallback_postmortem_review("code-656", "title-492");
        if item.move_to(FallbackPostmortemReviewStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPostmortemReviewStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_fallback_postmortem_review("code-656", "title-492");
        item.status = FallbackPostmortemReviewStatus::Closed;
        assert!(
            item.move_to(FallbackPostmortemReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
