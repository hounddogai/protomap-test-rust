use super::*;

use acme_kit::text;

/// Where a query record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryRecordStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryRecordStatus {
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

impl std::fmt::Display for QueryRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query record as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRecord {
    pub code: String,
    pub label: String,
    pub title: String,
    pub namespace: String,
    pub discount: f64,
    pub key_id: String,
    pub limit: i64,
    pub status: QueryRecordStatus,
}

impl QueryRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryRecordStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryRecordStatus) -> bool {
        match self.status {
            QueryRecordStatus::New => {
                return next == QueryRecordStatus::Failed;
            }
            QueryRecordStatus::Queued => {
                return next == QueryRecordStatus::Running || next == QueryRecordStatus::Failed;
            }
            QueryRecordStatus::Running => {
                return next == QueryRecordStatus::Done;
            }
            QueryRecordStatus::Done => {
                return next == QueryRecordStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "query record", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.limit as f64 - self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryRecord {
        QueryRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            namespace: self.namespace.clone(),
            discount: self.discount,
            key_id: self.key_id.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a query record in status new.
pub fn new_query_record(code: &str, label: &str) -> QueryRecord {
    QueryRecord {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        namespace: "".to_string(),
        discount: 0.0,
        key_id: "".to_string(),
        limit: 0,
        status: QueryRecordStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_query_record("code-182", "label-736");
        assert_eq!(item.status, QueryRecordStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_query_record("code-182", "label-736");
        if item.move_to(QueryRecordStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryRecordStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_query_record("code-182", "label-736");
        item.status = QueryRecordStatus::Failed;
        assert!(item.move_to(QueryRecordStatus::New).is_err(), "expected move to fail");
    }
}
