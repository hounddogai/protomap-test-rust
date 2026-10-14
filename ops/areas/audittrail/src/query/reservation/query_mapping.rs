use super::*;

use acme_kit::text;

/// Where a query mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryMappingStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QueryMappingStatus {
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

impl std::fmt::Display for QueryMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query mapping as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryMapping {
    pub code: String,
    pub name: String,
    pub source: String,
    pub reviewer: String,
    pub expires_at: i64,
    pub status: QueryMappingStatus,
}

impl QueryMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryMappingStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryMappingStatus) -> bool {
        match self.status {
            QueryMappingStatus::Planned => {
                return next == QueryMappingStatus::Cancelled;
            }
            QueryMappingStatus::Scheduled => {
                match next {
                    QueryMappingStatus::InProgress | QueryMappingStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryMappingStatus::InProgress => {
                return next == QueryMappingStatus::Completed || next == QueryMappingStatus::Cancelled;
            }
            QueryMappingStatus::Completed => {
                match next {
                    QueryMappingStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: QueryMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query mapping {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> QueryMapping {
        QueryMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            reviewer: self.reviewer.clone(),
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a query mapping in status planned.
pub fn new_query_mapping(code: &str, name: &str) -> QueryMapping {
    QueryMapping {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        reviewer: "".to_string(),
        expires_at: 0,
        status: QueryMappingStatus::Planned,
    }
}
