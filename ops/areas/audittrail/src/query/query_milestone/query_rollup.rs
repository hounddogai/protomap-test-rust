use super::*;

use acme_kit::text;

/// Where a query rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryRollupStatus {
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

impl std::fmt::Display for QueryRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRollup {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub tags: Vec<String>,
    pub active: bool,
    pub approved: bool,
    pub segment: String,
    pub rotation_days: i64,
    pub cost: f64,
    pub margin: f64,
    pub status: QueryRollupStatus,
}

impl QueryRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryRollupStatus) -> bool {
        match self.status {
            QueryRollupStatus::New => {
                match next {
                    QueryRollupStatus::Queued | QueryRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryRollupStatus::Queued => {
                return next == QueryRollupStatus::Failed;
            }
            QueryRollupStatus::Running => {
                return next == QueryRollupStatus::Failed;
            }
            QueryRollupStatus::Done => {
                match next {
                    QueryRollupStatus::Failed => {
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

    pub fn move_to(&mut self, next: QueryRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        (self.cost * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryRollup {
        QueryRollup {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            tags: self.tags.clone(),
            active: self.active,
            approved: self.approved,
            segment: self.segment.clone(),
            rotation_days: self.rotation_days,
            cost: self.cost,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a query rollup in status new.
pub fn new_query_rollup(code: &str, title: &str) -> QueryRollup {
    QueryRollup {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        tags: Vec::new(),
        active: false,
        approved: false,
        segment: "".to_string(),
        rotation_days: 0,
        cost: 0.0,
        margin: 0.0,
        status: QueryRollupStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_query_rollup("code-705", "title-432");
        assert_eq!(item.status, QueryRollupStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_query_rollup("code-705", "title-432");
        if item.move_to(QueryRollupStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryRollupStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_query_rollup("code-705", "title-432");
        item.status = QueryRollupStatus::Failed;
        assert!(item.move_to(QueryRollupStatus::New).is_err(), "expected move to fail");
    }
}
