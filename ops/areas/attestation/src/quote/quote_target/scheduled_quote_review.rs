use super::*;

use acme_kit::text;

/// Where a scheduled quote review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledQuoteReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledQuoteReviewStatus {
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

impl std::fmt::Display for ScheduledQuoteReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled quote review as the quote team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledQuoteReview {
    pub code: String,
    pub label: String,
    pub country: String,
    pub ends_at: i64,
    pub fingerprint: String,
    pub version: i64,
    pub duration_minutes: i64,
    pub external_ref: String,
    pub source: String,
    pub region: String,
    pub status: ScheduledQuoteReviewStatus,
}

impl ScheduledQuoteReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledQuoteReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledQuoteReviewStatus) -> bool {
        match self.status {
            ScheduledQuoteReviewStatus::Open => {
                return next == ScheduledQuoteReviewStatus::InReview || next == ScheduledQuoteReviewStatus::Resolved;
            }
            ScheduledQuoteReviewStatus::InReview => {
                return next == ScheduledQuoteReviewStatus::Reopened;
            }
            ScheduledQuoteReviewStatus::Resolved => {
                return next == ScheduledQuoteReviewStatus::Reopened || next == ScheduledQuoteReviewStatus::Archived;
            }
            ScheduledQuoteReviewStatus::Reopened => {
                match next {
                    ScheduledQuoteReviewStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledQuoteReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled quote review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.version as f64) * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledQuoteReview {
        ScheduledQuoteReview {
            code: self.code.clone(),
            label: self.label.clone(),
            country: self.country.clone(),
            ends_at: self.ends_at,
            fingerprint: self.fingerprint.clone(),
            version: self.version,
            duration_minutes: self.duration_minutes,
            external_ref: self.external_ref.clone(),
            source: self.source.clone(),
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled quote review in status open.
pub fn new_scheduled_quote_review(code: &str, label: &str) -> ScheduledQuoteReview {
    ScheduledQuoteReview {
        code: code.to_string(),
        label: label.to_string(),
        country: "".to_string(),
        ends_at: 0,
        fingerprint: "".to_string(),
        version: 0,
        duration_minutes: 0,
        external_ref: "".to_string(),
        source: "".to_string(),
        region: "".to_string(),
        status: ScheduledQuoteReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_quote_review("code-605", "label-588");
        assert_eq!(item.status, ScheduledQuoteReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_scheduled_quote_review("code-605", "label-588");
        if item.move_to(ScheduledQuoteReviewStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledQuoteReviewStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_quote_review("code-605", "label-588");
        item.status = ScheduledQuoteReviewStatus::Archived;
        assert!(
            item.move_to(ScheduledQuoteReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
