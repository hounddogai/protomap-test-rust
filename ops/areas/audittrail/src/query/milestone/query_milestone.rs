use super::*;

use acme_kit::text;

/// Where a query milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryMilestoneStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QueryMilestoneStatus {
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

impl std::fmt::Display for QueryMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query milestone as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryMilestone {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub score: f64,
    pub starts_at: i64,
    pub limit: i64,
    pub fingerprint: String,
    pub archived: bool,
    pub grace_days: i64,
    pub cost: f64,
    pub created_at: i64,
    pub status: QueryMilestoneStatus,
}

impl QueryMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryMilestoneStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryMilestoneStatus) -> bool {
        match self.status {
            QueryMilestoneStatus::Planned => {
                match next {
                    QueryMilestoneStatus::Scheduled | QueryMilestoneStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryMilestoneStatus::Scheduled => {
                match next {
                    QueryMilestoneStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryMilestoneStatus::InProgress => {
                match next {
                    QueryMilestoneStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryMilestoneStatus::Completed => {
                match next {
                    QueryMilestoneStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: QueryMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query milestone {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.score + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryMilestone {
        QueryMilestone {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            score: self.score,
            starts_at: self.starts_at,
            limit: self.limit,
            fingerprint: self.fingerprint.clone(),
            archived: self.archived,
            grace_days: self.grace_days,
            cost: self.cost,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query milestone in status planned.
pub fn new_query_milestone(code: &str, title: &str) -> QueryMilestone {
    QueryMilestone {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        score: 0.0,
        starts_at: 0,
        limit: 0,
        fingerprint: "".to_string(),
        archived: false,
        grace_days: 0,
        cost: 0.0,
        created_at: 0,
        status: QueryMilestoneStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_query_milestone("code-107", "title-518");
        assert_eq!(item.status, QueryMilestoneStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_query_milestone("code-107", "title-518");
        if item.move_to(QueryMilestoneStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryMilestoneStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_query_milestone("code-107", "title-518");
        item.status = QueryMilestoneStatus::Cancelled;
        assert!(
            item.move_to(QueryMilestoneStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
