use super::*;

use acme_kit::text;

/// Where a adaptive quote segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveQuoteSegmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveQuoteSegmentStatus {
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

impl std::fmt::Display for AdaptiveQuoteSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One adaptive quote segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveQuoteSegment {
    pub code: String,
    pub title: String,
    pub budget: f64,
    pub threshold: f64,
    pub contact_email: String,
    pub retries: i64,
    pub quota_units: i64,
    pub summary: String,
    pub pinned: bool,
    pub status: AdaptiveQuoteSegmentStatus,
}

impl AdaptiveQuoteSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveQuoteSegmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveQuoteSegmentStatus) -> bool {
        match self.status {
            AdaptiveQuoteSegmentStatus::Planned => {
                match next {
                    AdaptiveQuoteSegmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveQuoteSegmentStatus::Scheduled => {
                match next {
                    AdaptiveQuoteSegmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveQuoteSegmentStatus::InProgress => {
                return next == AdaptiveQuoteSegmentStatus::Cancelled;
            }
            AdaptiveQuoteSegmentStatus::Completed => {
                return next == AdaptiveQuoteSegmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveQuoteSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive quote segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive quote segment", self.code)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.threshold + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveQuoteSegment {
        AdaptiveQuoteSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            budget: self.budget,
            threshold: self.threshold,
            contact_email: self.contact_email.clone(),
            retries: self.retries,
            quota_units: self.quota_units,
            summary: self.summary.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a adaptive quote segment in status planned.
pub fn new_adaptive_quote_segment(code: &str, title: &str) -> AdaptiveQuoteSegment {
    AdaptiveQuoteSegment {
        code: code.to_string(),
        title: title.to_string(),
        budget: 0.0,
        threshold: 0.0,
        contact_email: "".to_string(),
        retries: 0,
        quota_units: 0,
        summary: "".to_string(),
        pinned: false,
        status: AdaptiveQuoteSegmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_adaptive_quote_segment("code-841", "title-291");
        assert_eq!(item.status, AdaptiveQuoteSegmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_adaptive_quote_segment("code-841", "title-291");
        if item.move_to(AdaptiveQuoteSegmentStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveQuoteSegmentStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_adaptive_quote_segment("code-841", "title-291");
        item.status = AdaptiveQuoteSegmentStatus::Cancelled;
        assert!(
            item.move_to(AdaptiveQuoteSegmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
