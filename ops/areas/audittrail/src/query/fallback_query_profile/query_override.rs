use super::*;

use acme_kit::text;

/// Where a query override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryOverrideStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl QueryOverrideStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for QueryOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query override. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryOverride {
    pub code: String,
    pub title: String,
    pub category: String,
    pub updated_at: i64,
    pub sequence: i64,
    pub status: QueryOverrideStatus,
}

impl QueryOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryOverrideStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryOverrideStatus) -> bool {
        match self.status {
            QueryOverrideStatus::Created => {
                match next {
                    QueryOverrideStatus::Published | QueryOverrideStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryOverrideStatus::Validated => {
                return next == QueryOverrideStatus::Published;
            }
            QueryOverrideStatus::Published => {
                return next == QueryOverrideStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query override", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.updated_at as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryOverride {
        QueryOverride {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            updated_at: self.updated_at,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a query override in status created.
pub fn new_query_override(code: &str, title: &str) -> QueryOverride {
    QueryOverride {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        updated_at: 0,
        sequence: 0,
        status: QueryOverrideStatus::Created,
    }
}
