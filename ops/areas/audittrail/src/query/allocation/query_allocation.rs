use super::*;

use acme_kit::text;

/// Where a query allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryAllocationStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QueryAllocationStatus {
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

impl std::fmt::Display for QueryAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAllocation {
    pub code: String,
    pub name: String,
    pub ends_at: i64,
    pub scope: String,
    pub archived: bool,
    pub priority: i64,
    pub margin: f64,
    pub issuer: String,
    pub grace_days: i64,
    pub status: QueryAllocationStatus,
}

impl QueryAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryAllocationStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryAllocationStatus) -> bool {
        match self.status {
            QueryAllocationStatus::Open => {
                return next == QueryAllocationStatus::Reopened || next == QueryAllocationStatus::Archived;
            }
            QueryAllocationStatus::InReview => {
                match next {
                    QueryAllocationStatus::Resolved | QueryAllocationStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryAllocationStatus::Resolved => {
                match next {
                    QueryAllocationStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryAllocationStatus::Reopened => {
                return next == QueryAllocationStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query allocation", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.margin - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryAllocation {
        QueryAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            ends_at: self.ends_at,
            scope: self.scope.clone(),
            archived: self.archived,
            priority: self.priority,
            margin: self.margin,
            issuer: self.issuer.clone(),
            grace_days: self.grace_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a query allocation in status open.
pub fn new_query_allocation(code: &str, name: &str) -> QueryAllocation {
    QueryAllocation {
        code: code.to_string(),
        name: name.to_string(),
        ends_at: 0,
        scope: "".to_string(),
        archived: false,
        priority: 0,
        margin: 0.0,
        issuer: "".to_string(),
        grace_days: 0,
        status: QueryAllocationStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_query_allocation("code-804", "name-268");
        assert_eq!(item.status, QueryAllocationStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_query_allocation("code-804", "name-268");
        if item.move_to(QueryAllocationStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryAllocationStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_query_allocation("code-804", "name-268");
        item.status = QueryAllocationStatus::Archived;
        assert!(
            item.move_to(QueryAllocationStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
