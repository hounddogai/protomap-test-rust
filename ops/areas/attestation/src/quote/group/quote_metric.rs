use super::*;

use acme_kit::text;

/// Where a quote metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteMetricStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QuoteMetricStatus {
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

impl std::fmt::Display for QuoteMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteMetric {
    pub code: String,
    pub title: String,
    pub scope: String,
    pub pinned: bool,
    pub grace_days: i64,
    pub status: QuoteMetricStatus,
}

impl QuoteMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteMetricStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteMetricStatus) -> bool {
        match self.status {
            QuoteMetricStatus::Planned => {
                return next == QuoteMetricStatus::Scheduled || next == QuoteMetricStatus::Completed;
            }
            QuoteMetricStatus::Scheduled => {
                match next {
                    QuoteMetricStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteMetricStatus::InProgress => {
                return next == QuoteMetricStatus::Cancelled;
            }
            QuoteMetricStatus::Completed => {
                return next == QuoteMetricStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.grace_days as f64
    }

    pub fn copy(&self) -> QuoteMetric {
        QuoteMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            scope: self.scope.clone(),
            pinned: self.pinned,
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a quote metric in status planned.
pub fn new_quote_metric(code: &str, title: &str) -> QuoteMetric {
    QuoteMetric {
        code: code.to_string(),
        title: title.to_string(),
        scope: "".to_string(),
        pinned: false,
        grace_days: 0,
        status: QuoteMetricStatus::Planned,
    }
}
