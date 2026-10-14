use super::*;

use acme_kit::text;

/// Where a query schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryScheduleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QueryScheduleStatus {
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

impl std::fmt::Display for QueryScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query schedule as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuerySchedule {
    pub code: String,
    pub label: String,
    pub priority: i64,
    pub threshold: f64,
    pub cipher: String,
    pub starts_at: i64,
    pub notes: String,
    pub status: QueryScheduleStatus,
}

impl QuerySchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryScheduleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryScheduleStatus) -> bool {
        match self.status {
            QueryScheduleStatus::Draft => {
                match next {
                    QueryScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryScheduleStatus::Pending => {
                match next {
                    QueryScheduleStatus::Active | QueryScheduleStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryScheduleStatus::Active => {
                match next {
                    QueryScheduleStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryScheduleStatus::Suspended => {
                return next == QueryScheduleStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query schedule {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} {}", "query schedule", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.threshold - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuerySchedule {
        QuerySchedule {
            code: self.code.clone(),
            label: self.label.clone(),
            priority: self.priority,
            threshold: self.threshold,
            cipher: self.cipher.clone(),
            starts_at: self.starts_at,
            notes: self.notes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a query schedule in status draft.
pub fn new_query_schedule(code: &str, label: &str) -> QuerySchedule {
    QuerySchedule {
        code: code.to_string(),
        label: label.to_string(),
        priority: 0,
        threshold: 0.0,
        cipher: "".to_string(),
        starts_at: 0,
        notes: "".to_string(),
        status: QueryScheduleStatus::Draft,
    }
}
