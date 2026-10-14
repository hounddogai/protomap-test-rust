use super::*;

use acme_kit::text;

/// Where a query ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryLedgerStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl QueryLedgerStatus {
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

impl std::fmt::Display for QueryLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryLedger {
    pub code: String,
    pub label: String,
    pub retries: i64,
    pub notes: String,
    pub unit: String,
    pub ends_at: i64,
    pub owner_id: String,
    pub contact_email: String,
    pub status: QueryLedgerStatus,
}

impl QueryLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryLedgerStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryLedgerStatus) -> bool {
        match self.status {
            QueryLedgerStatus::Open => {
                match next {
                    QueryLedgerStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryLedgerStatus::InReview => {
                match next {
                    QueryLedgerStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryLedgerStatus::Resolved => {
                return next == QueryLedgerStatus::Reopened || next == QueryLedgerStatus::Archived;
            }
            QueryLedgerStatus::Reopened => {
                return next == QueryLedgerStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query ledger {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ends_at as f64) * (self.retries as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryLedger {
        QueryLedger {
            code: self.code.clone(),
            label: self.label.clone(),
            retries: self.retries,
            notes: self.notes.clone(),
            unit: self.unit.clone(),
            ends_at: self.ends_at,
            owner_id: self.owner_id.clone(),
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a query ledger in status open.
pub fn new_query_ledger(code: &str, label: &str) -> QueryLedger {
    QueryLedger {
        code: code.to_string(),
        label: label.to_string(),
        retries: 0,
        notes: "".to_string(),
        unit: "".to_string(),
        ends_at: 0,
        owner_id: "".to_string(),
        contact_email: "".to_string(),
        status: QueryLedgerStatus::Open,
    }
}
