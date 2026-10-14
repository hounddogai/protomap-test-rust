use super::*;

use acme_kit::text;

/// Where a query manifest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryManifestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QueryManifestStatus {
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

impl std::fmt::Display for QueryManifestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query manifest. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryManifest {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub floor_value: f64,
    pub country: String,
    pub channel: String,
    pub burst: i64,
    pub rate: f64,
    pub retries: i64,
    pub threshold: f64,
    pub status: QueryManifestStatus,
}

impl QueryManifest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryManifestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryManifestStatus) -> bool {
        match self.status {
            QueryManifestStatus::Open => {
                return next == QueryManifestStatus::Resolved;
            }
            QueryManifestStatus::InReview => {
                return next == QueryManifestStatus::Resolved || next == QueryManifestStatus::Reopened;
            }
            QueryManifestStatus::Resolved => {
                match next {
                    QueryManifestStatus::Reopened | QueryManifestStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryManifestStatus::Reopened => {
                return next == QueryManifestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryManifestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query manifest {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.updated_at as f64 - self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryManifest {
        QueryManifest {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            floor_value: self.floor_value,
            country: self.country.clone(),
            channel: self.channel.clone(),
            burst: self.burst,
            rate: self.rate,
            retries: self.retries,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a query manifest in status open.
pub fn new_query_manifest(code: &str, title: &str) -> QueryManifest {
    QueryManifest {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        floor_value: 0.0,
        country: "".to_string(),
        channel: "".to_string(),
        burst: 0,
        rate: 0.0,
        retries: 0,
        threshold: 0.0,
        status: QueryManifestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_query_manifest("code-577", "title-897");
        assert_eq!(item.status, QueryManifestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_query_manifest("code-577", "title-897");
        if item.move_to(QueryManifestStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryManifestStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_query_manifest("code-577", "title-897");
        item.status = QueryManifestStatus::Archived;
        assert!(
            item.move_to(QueryManifestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
