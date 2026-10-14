use super::*;

use acme_kit::text;

/// Where a query exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryExceptionStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QueryExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for QueryExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a query exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryException {
    pub code: String,
    pub title: String,
    pub approved: bool,
    pub contact_email: String,
    pub cost: f64,
    pub status: QueryExceptionStatus,
}

impl QueryException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryExceptionStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryExceptionStatus) -> bool {
        match self.status {
            QueryExceptionStatus::Draft => {
                return next == QueryExceptionStatus::Pending;
            }
            QueryExceptionStatus::Pending => {
                return next == QueryExceptionStatus::Suspended;
            }
            QueryExceptionStatus::Active => {
                return next == QueryExceptionStatus::Suspended || next == QueryExceptionStatus::Closed;
            }
            QueryExceptionStatus::Suspended => {
                return next == QueryExceptionStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query exception", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> QueryException {
        QueryException {
            code: self.code.clone(),
            title: self.title.clone(),
            approved: self.approved,
            contact_email: self.contact_email.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query exception in status draft.
pub fn new_query_exception(code: &str, title: &str) -> QueryException {
    QueryException {
        code: code.to_string(),
        title: title.to_string(),
        approved: false,
        contact_email: "".to_string(),
        cost: 0.0,
        status: QueryExceptionStatus::Draft,
    }
}
