use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a query request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryRequestStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QueryRequestStatus {
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

impl std::fmt::Display for QueryRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query request as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryRequest {
    pub code: String,
    pub label: String,
    pub reviewer: String,
    pub weight: f64,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub capacity: i64,
    pub batch_size: i64,
    pub limit: i64,
    pub tags: Vec<String>,
    pub status: QueryRequestStatus,
}

impl QueryRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryRequestStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryRequestStatus) -> bool {
        match self.status {
            QueryRequestStatus::Draft => {
                match next {
                    QueryRequestStatus::Pending | QueryRequestStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryRequestStatus::Pending => {
                return next == QueryRequestStatus::Active || next == QueryRequestStatus::Suspended;
            }
            QueryRequestStatus::Active => {
                match next {
                    QueryRequestStatus::Suspended | QueryRequestStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryRequestStatus::Suspended => {
                match next {
                    QueryRequestStatus::Closed => {
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

    pub fn move_to(&mut self, next: QueryRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "query request", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.limit as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryRequest {
        QueryRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            reviewer: self.reviewer.clone(),
            weight: self.weight,
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            capacity: self.capacity,
            batch_size: self.batch_size,
            limit: self.limit,
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a query request in status draft.
pub fn new_query_request(code: &str, label: &str) -> QueryRequest {
    QueryRequest {
        code: code.to_string(),
        label: label.to_string(),
        reviewer: "".to_string(),
        weight: 0.0,
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        capacity: 0,
        batch_size: 0,
        limit: 0,
        tags: Vec::new(),
        status: QueryRequestStatus::Draft,
    }
}
