use super::*;

use acme_kit::text;

/// Where a query entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryEntryStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QueryEntryStatus {
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

impl std::fmt::Display for QueryEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEntry {
    pub code: String,
    pub name: String,
    pub visible: bool,
    pub unit: String,
    pub external_ref: String,
    pub principal: String,
    pub issuer: String,
    pub quantity: i64,
    pub status: QueryEntryStatus,
}

impl QueryEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryEntryStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryEntryStatus) -> bool {
        match self.status {
            QueryEntryStatus::Open => {
                match next {
                    QueryEntryStatus::InReview | QueryEntryStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryEntryStatus::InReview => {
                match next {
                    QueryEntryStatus::Resolved | QueryEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryEntryStatus::Resolved => {
                match next {
                    QueryEntryStatus::Reopened | QueryEntryStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryEntryStatus::Reopened => {
                return next == QueryEntryStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> QueryEntry {
        QueryEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            visible: self.visible,
            unit: self.unit.clone(),
            external_ref: self.external_ref.clone(),
            principal: self.principal.clone(),
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a query entry in status open.
pub fn new_query_entry(code: &str, name: &str) -> QueryEntry {
    QueryEntry {
        code: code.to_string(),
        name: name.to_string(),
        visible: false,
        unit: "".to_string(),
        external_ref: "".to_string(),
        principal: "".to_string(),
        issuer: "".to_string(),
        quantity: 0,
        status: QueryEntryStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_query_entry("code-415", "name-290");
        assert_eq!(item.status, QueryEntryStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_query_entry("code-415", "name-290");
        if item.move_to(QueryEntryStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryEntryStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_query_entry("code-415", "name-290");
        item.status = QueryEntryStatus::Archived;
        assert!(item.move_to(QueryEntryStatus::Open).is_err(), "expected move to fail");
    }
}
