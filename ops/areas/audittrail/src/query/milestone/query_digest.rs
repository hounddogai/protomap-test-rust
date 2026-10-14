use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a query digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryDigestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryDigestStatus {
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

impl std::fmt::Display for QueryDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query digest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryDigest {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub maximum: f64,
    pub attributes: BTreeMap<String, String>,
    pub duration_minutes: i64,
    pub scope: String,
    pub quota_units: i64,
    pub status: QueryDigestStatus,
}

impl QueryDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryDigestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryDigestStatus) -> bool {
        match self.status {
            QueryDigestStatus::New => {
                match next {
                    QueryDigestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryDigestStatus::Queued => {
                match next {
                    QueryDigestStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryDigestStatus::Running => {
                return next == QueryDigestStatus::Done || next == QueryDigestStatus::Failed;
            }
            QueryDigestStatus::Done => {
                return next == QueryDigestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_load(&self) -> f64 {
        ((self.quota_units as f64) * self.maximum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryDigest {
        QueryDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            maximum: self.maximum,
            attributes: self.attributes.clone(),
            duration_minutes: self.duration_minutes,
            scope: self.scope.clone(),
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a query digest in status new.
pub fn new_query_digest(code: &str, title: &str) -> QueryDigest {
    QueryDigest {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        maximum: 0.0,
        attributes: BTreeMap::new(),
        duration_minutes: 0,
        scope: "".to_string(),
        quota_units: 0,
        status: QueryDigestStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_query_digest("code-20", "title-731");
        assert_eq!(item.status, QueryDigestStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_query_digest("code-20", "title-731");
        if item.move_to(QueryDigestStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryDigestStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_query_digest("code-20", "title-731");
        item.status = QueryDigestStatus::Failed;
        assert!(item.move_to(QueryDigestStatus::New).is_err(), "expected move to fail");
    }
}
