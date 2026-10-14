use super::*;

use acme_kit::text;

/// Where a query draft is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryDraftStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl QueryDraftStatus {
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

impl std::fmt::Display for QueryDraftStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query draft and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryDraft {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub pinned: bool,
    pub source: String,
    pub description: String,
    pub visible: bool,
    pub principal: String,
    pub duration_minutes: i64,
    pub budget: f64,
    pub margin: f64,
    pub status: QueryDraftStatus,
}

impl QueryDraft {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryDraftStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryDraftStatus) -> bool {
        match self.status {
            QueryDraftStatus::Planned => {
                match next {
                    QueryDraftStatus::Scheduled | QueryDraftStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryDraftStatus::Scheduled => {
                match next {
                    QueryDraftStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryDraftStatus::InProgress => {
                return next == QueryDraftStatus::Completed || next == QueryDraftStatus::Cancelled;
            }
            QueryDraftStatus::Completed => {
                match next {
                    QueryDraftStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: QueryDraftStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query draft {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.ceiling + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryDraft {
        QueryDraft {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            pinned: self.pinned,
            source: self.source.clone(),
            description: self.description.clone(),
            visible: self.visible,
            principal: self.principal.clone(),
            duration_minutes: self.duration_minutes,
            budget: self.budget,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a query draft in status planned.
pub fn new_query_draft(code: &str, title: &str) -> QueryDraft {
    QueryDraft {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        pinned: false,
        source: "".to_string(),
        description: "".to_string(),
        visible: false,
        principal: "".to_string(),
        duration_minutes: 0,
        budget: 0.0,
        margin: 0.0,
        status: QueryDraftStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_query_draft("code-881", "title-267");
        assert_eq!(item.status, QueryDraftStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_query_draft("code-881", "title-267");
        if item.move_to(QueryDraftStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryDraftStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_query_draft("code-881", "title-267");
        item.status = QueryDraftStatus::Cancelled;
        assert!(
            item.move_to(QueryDraftStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
