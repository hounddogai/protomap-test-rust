use super::*;

use acme_kit::text;

/// Where a query batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryBatchStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryBatchStatus {
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

impl std::fmt::Display for QueryBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query batch. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryBatch {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub contact_email: String,
    pub attempts: i64,
    pub threshold: f64,
    pub namespace: String,
    pub status: QueryBatchStatus,
}

impl QueryBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryBatchStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryBatchStatus) -> bool {
        match self.status {
            QueryBatchStatus::New => {
                match next {
                    QueryBatchStatus::Done | QueryBatchStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryBatchStatus::Queued => {
                return next == QueryBatchStatus::Running || next == QueryBatchStatus::Done;
            }
            QueryBatchStatus::Running => {
                match next {
                    QueryBatchStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryBatchStatus::Done => {
                return next == QueryBatchStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.attempts as f64 + self.threshold) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryBatch {
        QueryBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            contact_email: self.contact_email.clone(),
            attempts: self.attempts,
            threshold: self.threshold,
            namespace: self.namespace.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a query batch in status new.
pub fn new_query_batch(code: &str, label: &str) -> QueryBatch {
    QueryBatch {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        contact_email: "".to_string(),
        attempts: 0,
        threshold: 0.0,
        namespace: "".to_string(),
        status: QueryBatchStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_query_batch("code-826", "label-20");
        assert_eq!(item.status, QueryBatchStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_query_batch("code-826", "label-20");
        if item.move_to(QueryBatchStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryBatchStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_query_batch("code-826", "label-20");
        item.status = QueryBatchStatus::Failed;
        assert!(item.move_to(QueryBatchStatus::New).is_err(), "expected move to fail");
    }
}
