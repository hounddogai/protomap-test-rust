use super::*;

use acme_kit::text;

/// Where a rolling reservation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingReservationSummaryStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl RollingReservationSummaryStatus {
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

impl std::fmt::Display for RollingReservationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling reservation summary. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingReservationSummary {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub budget: f64,
    pub ends_at: i64,
    pub timezone: String,
    pub ceiling: f64,
    pub source: String,
    pub status: RollingReservationSummaryStatus,
}

impl RollingReservationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingReservationSummaryStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingReservationSummaryStatus) -> bool {
        match self.status {
            RollingReservationSummaryStatus::Planned => {
                return next == RollingReservationSummaryStatus::Completed;
            }
            RollingReservationSummaryStatus::Scheduled => {
                return next == RollingReservationSummaryStatus::InProgress
                    || next == RollingReservationSummaryStatus::Cancelled;
            }
            RollingReservationSummaryStatus::InProgress => {
                return next == RollingReservationSummaryStatus::Completed;
            }
            RollingReservationSummaryStatus::Completed => {
                return next == RollingReservationSummaryStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingReservationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling reservation summary {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.ceiling * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingReservationSummary {
        RollingReservationSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            budget: self.budget,
            ends_at: self.ends_at,
            timezone: self.timezone.clone(),
            ceiling: self.ceiling,
            source: self.source.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling reservation summary in status planned.
pub fn new_rolling_reservation_summary(code: &str, title: &str) -> RollingReservationSummary {
    RollingReservationSummary {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        budget: 0.0,
        ends_at: 0,
        timezone: "".to_string(),
        ceiling: 0.0,
        source: "".to_string(),
        status: RollingReservationSummaryStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_rolling_reservation_summary("code-127", "title-82");
        assert_eq!(item.status, RollingReservationSummaryStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_rolling_reservation_summary("code-127", "title-82");
        if item.move_to(RollingReservationSummaryStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingReservationSummaryStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_rolling_reservation_summary("code-127", "title-82");
        item.status = RollingReservationSummaryStatus::Cancelled;
        assert!(
            item.move_to(RollingReservationSummaryStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
