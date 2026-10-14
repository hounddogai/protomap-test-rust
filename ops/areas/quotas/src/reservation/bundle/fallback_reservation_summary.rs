use super::*;

use acme_kit::text;

/// Where a fallback reservation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackReservationSummaryStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackReservationSummaryStatus {
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

impl std::fmt::Display for FallbackReservationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback reservation summary, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackReservationSummary {
    pub code: String,
    pub title: String,
    pub limit: i64,
    pub capacity: i64,
    pub locale: String,
    pub unit: String,
    pub score: f64,
    pub visible: bool,
    pub port: i64,
    pub status: FallbackReservationSummaryStatus,
}

impl FallbackReservationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackReservationSummaryStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackReservationSummaryStatus) -> bool {
        match self.status {
            FallbackReservationSummaryStatus::New => {
                return next == FallbackReservationSummaryStatus::Running
                    || next == FallbackReservationSummaryStatus::Failed;
            }
            FallbackReservationSummaryStatus::Queued => {
                match next {
                    FallbackReservationSummaryStatus::Running | FallbackReservationSummaryStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackReservationSummaryStatus::Running => {
                return next == FallbackReservationSummaryStatus::Done;
            }
            FallbackReservationSummaryStatus::Done => {
                return next == FallbackReservationSummaryStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackReservationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback reservation summary {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "fallback reservation summary", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.port as f64 - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackReservationSummary {
        FallbackReservationSummary {
            code: self.code.clone(),
            title: self.title.clone(),
            limit: self.limit,
            capacity: self.capacity,
            locale: self.locale.clone(),
            unit: self.unit.clone(),
            score: self.score,
            visible: self.visible,
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a fallback reservation summary in status new.
pub fn new_fallback_reservation_summary(code: &str, title: &str) -> FallbackReservationSummary {
    FallbackReservationSummary {
        code: code.to_string(),
        title: title.to_string(),
        limit: 0,
        capacity: 0,
        locale: "".to_string(),
        unit: "".to_string(),
        score: 0.0,
        visible: false,
        port: 0,
        status: FallbackReservationSummaryStatus::New,
    }
}
