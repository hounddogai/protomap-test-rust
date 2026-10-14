use super::*;

use acme_kit::text;

/// Where a query assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QueryAssignmentStatus {
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

impl std::fmt::Display for QueryAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query assignment. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAssignment {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub checksum: String,
    pub external_ref: String,
    pub duration_minutes: i64,
    pub port: i64,
    pub currency: String,
    pub issuer: String,
    pub phone: String,
    pub status: QueryAssignmentStatus,
}

impl QueryAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryAssignmentStatus) -> bool {
        match self.status {
            QueryAssignmentStatus::Open => {
                return next == QueryAssignmentStatus::Reopened;
            }
            QueryAssignmentStatus::InReview => {
                return next == QueryAssignmentStatus::Reopened || next == QueryAssignmentStatus::Archived;
            }
            QueryAssignmentStatus::Resolved => {
                match next {
                    QueryAssignmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryAssignmentStatus::Reopened => {
                match next {
                    QueryAssignmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: QueryAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.batch_size as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryAssignment {
        QueryAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            checksum: self.checksum.clone(),
            external_ref: self.external_ref.clone(),
            duration_minutes: self.duration_minutes,
            port: self.port,
            currency: self.currency.clone(),
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a query assignment in status open.
pub fn new_query_assignment(code: &str, name: &str) -> QueryAssignment {
    QueryAssignment {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        checksum: "".to_string(),
        external_ref: "".to_string(),
        duration_minutes: 0,
        port: 0,
        currency: "".to_string(),
        issuer: "".to_string(),
        phone: "".to_string(),
        status: QueryAssignmentStatus::Open,
    }
}
