use super::*;

use acme_kit::text;

/// Where a query threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryThresholdStatus {
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

impl std::fmt::Display for QueryThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryThreshold {
    pub code: String,
    pub title: String,
    pub country: String,
    pub approved: bool,
    pub starts_at: i64,
    pub capacity: i64,
    pub window_days: i64,
    pub channel: String,
    pub fingerprint: String,
    pub status: QueryThresholdStatus,
}

impl QueryThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryThresholdStatus) -> bool {
        match self.status {
            QueryThresholdStatus::New => {
                return next == QueryThresholdStatus::Done;
            }
            QueryThresholdStatus::Queued => {
                match next {
                    QueryThresholdStatus::Done | QueryThresholdStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryThresholdStatus::Running => {
                return next == QueryThresholdStatus::Done;
            }
            QueryThresholdStatus::Done => {
                match next {
                    QueryThresholdStatus::Failed => {
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

    pub fn move_to(&mut self, next: QueryThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query threshold {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.capacity as f64 + (self.window_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryThreshold {
        QueryThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            country: self.country.clone(),
            approved: self.approved,
            starts_at: self.starts_at,
            capacity: self.capacity,
            window_days: self.window_days,
            channel: self.channel.clone(),
            fingerprint: self.fingerprint.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a query threshold in status new.
pub fn new_query_threshold(code: &str, title: &str) -> QueryThreshold {
    QueryThreshold {
        code: code.to_string(),
        title: title.to_string(),
        country: "".to_string(),
        approved: false,
        starts_at: 0,
        capacity: 0,
        window_days: 0,
        channel: "".to_string(),
        fingerprint: "".to_string(),
        status: QueryThresholdStatus::New,
    }
}
