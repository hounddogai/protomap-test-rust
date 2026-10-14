use super::*;

use acme_kit::text;

/// Where a scheduled reservation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledReservationSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledReservationSummaryStatus {
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

impl std::fmt::Display for ScheduledReservationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled reservation summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledReservationSummary {
    pub code: String,
    pub name: String,
    pub port: i64,
    pub burst: i64,
    pub quota_units: i64,
    pub window_days: i64,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub country: String,
    pub visible: bool,
    pub ratio: f64,
    pub status: ScheduledReservationSummaryStatus,
}

impl ScheduledReservationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledReservationSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledReservationSummaryStatus) -> bool {
        match self.status {
            ScheduledReservationSummaryStatus::Open => {
                match next {
                    ScheduledReservationSummaryStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledReservationSummaryStatus::InReview => {
                match next {
                    ScheduledReservationSummaryStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledReservationSummaryStatus::Resolved => {
                return next == ScheduledReservationSummaryStatus::Archived;
            }
            ScheduledReservationSummaryStatus::Reopened => {
                return next == ScheduledReservationSummaryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledReservationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled reservation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled reservation summary", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_total(&self) -> f64 {
        (self.threshold * self.ratio * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledReservationSummary {
        ScheduledReservationSummary {
            code: self.code.clone(),
            name: self.name.clone(),
            port: self.port,
            burst: self.burst,
            quota_units: self.quota_units,
            window_days: self.window_days,
            threshold: self.threshold,
            tags: self.tags.clone(),
            country: self.country.clone(),
            visible: self.visible,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a scheduled reservation summary in status open.
pub fn new_scheduled_reservation_summary(code: &str, name: &str) -> ScheduledReservationSummary {
    ScheduledReservationSummary {
        code: code.to_string(),
        name: name.to_string(),
        port: 0,
        burst: 0,
        quota_units: 0,
        window_days: 0,
        threshold: 0.0,
        tags: Vec::new(),
        country: "".to_string(),
        visible: false,
        ratio: 0.0,
        status: ScheduledReservationSummaryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_reservation_summary("code-488", "name-527");
        assert_eq!(item.status, ScheduledReservationSummaryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_scheduled_reservation_summary("code-488", "name-527");
        if item.move_to(ScheduledReservationSummaryStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledReservationSummaryStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_reservation_summary("code-488", "name-527");
        item.status = ScheduledReservationSummaryStatus::Archived;
        assert!(
            item.move_to(ScheduledReservationSummaryStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
