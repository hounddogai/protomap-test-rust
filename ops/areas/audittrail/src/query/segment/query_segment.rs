use super::*;

use acme_kit::text;

/// Where a query segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuerySegmentStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QuerySegmentStatus {
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

impl std::fmt::Display for QuerySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuerySegment {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub minimum: f64,
    pub duration_minutes: i64,
    pub port: i64,
    pub expires_at: i64,
    pub score: f64,
    pub summary: String,
    pub status: QuerySegmentStatus,
}

impl QuerySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuerySegmentStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuerySegmentStatus) -> bool {
        match self.status {
            QuerySegmentStatus::New => {
                match next {
                    QuerySegmentStatus::Queued => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuerySegmentStatus::Queued => {
                return next == QuerySegmentStatus::Running;
            }
            QuerySegmentStatus::Running => {
                return next == QuerySegmentStatus::Done || next == QuerySegmentStatus::Failed;
            }
            QuerySegmentStatus::Done => {
                return next == QuerySegmentStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuerySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        (self.discount * (self.expires_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuerySegment {
        QuerySegment {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            minimum: self.minimum,
            duration_minutes: self.duration_minutes,
            port: self.port,
            expires_at: self.expires_at,
            score: self.score,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a query segment in status new.
pub fn new_query_segment(code: &str, label: &str) -> QuerySegment {
    QuerySegment {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        minimum: 0.0,
        duration_minutes: 0,
        port: 0,
        expires_at: 0,
        score: 0.0,
        summary: "".to_string(),
        status: QuerySegmentStatus::New,
    }
}
