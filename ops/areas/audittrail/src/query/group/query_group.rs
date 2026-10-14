use super::*;

use acme_kit::text;

/// Where a query group is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryGroupStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QueryGroupStatus {
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

impl std::fmt::Display for QueryGroupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query group, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryGroup {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub minimum: f64,
    pub unit: String,
    pub rotation_days: i64,
    pub status: QueryGroupStatus,
}

impl QueryGroup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryGroupStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryGroupStatus) -> bool {
        match self.status {
            QueryGroupStatus::Open => {
                match next {
                    QueryGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryGroupStatus::InReview => {
                return next == QueryGroupStatus::Archived;
            }
            QueryGroupStatus::Resolved => {
                match next {
                    QueryGroupStatus::Reopened | QueryGroupStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryGroupStatus::Reopened => {
                return next == QueryGroupStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryGroupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query group {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query group", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.minimum - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryGroup {
        QueryGroup {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            minimum: self.minimum,
            unit: self.unit.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a query group in status open.
pub fn new_query_group(code: &str, name: &str) -> QueryGroup {
    QueryGroup {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        minimum: 0.0,
        unit: "".to_string(),
        rotation_days: 0,
        status: QueryGroupStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_query_group("code-51", "name-198");
        assert_eq!(item.status, QueryGroupStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_query_group("code-51", "name-198");
        if item.move_to(QueryGroupStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryGroupStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_query_group("code-51", "name-198");
        item.status = QueryGroupStatus::Archived;
        assert!(item.move_to(QueryGroupStatus::Open).is_err(), "expected move to fail");
    }
}
