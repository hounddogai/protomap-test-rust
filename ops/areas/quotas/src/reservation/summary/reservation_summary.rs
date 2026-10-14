use super::*;

use acme_kit::text;

/// Where a reservation summary is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationSummaryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReservationSummaryStatus {
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

impl std::fmt::Display for ReservationSummaryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A reservation summary as the reservation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationSummary {
    pub code: String,
    pub label: String,
    pub description: String,
    pub version: i64,
    pub audience: String,
    pub limit: i64,
    pub ends_at: i64,
    pub notes: String,
    pub status: ReservationSummaryStatus,
}

impl ReservationSummary {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationSummaryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationSummaryStatus) -> bool {
        match self.status {
            ReservationSummaryStatus::Open => {
                match next {
                    ReservationSummaryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationSummaryStatus::InReview => {
                return next == ReservationSummaryStatus::Resolved;
            }
            ReservationSummaryStatus::Resolved => {
                match next {
                    ReservationSummaryStatus::Reopened | ReservationSummaryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationSummaryStatus::Reopened => {
                match next {
                    ReservationSummaryStatus::Archived => {
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

    pub fn move_to(&mut self, next: ReservationSummaryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation summary {} cannot move from {} to {}",
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
        format!("{} {}", "reservation summary", self.code)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.limit as f64 + (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationSummary {
        ReservationSummary {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            version: self.version,
            audience: self.audience.clone(),
            limit: self.limit,
            ends_at: self.ends_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a reservation summary in status open.
pub fn new_reservation_summary(code: &str, label: &str) -> ReservationSummary {
    ReservationSummary {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        version: 0,
        audience: "".to_string(),
        limit: 0,
        ends_at: 0,
        notes: "".to_string(),
        status: ReservationSummaryStatus::Open,
    }
}
