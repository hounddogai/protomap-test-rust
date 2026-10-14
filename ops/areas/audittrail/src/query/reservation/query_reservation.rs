use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a query reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryReservationStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryReservationStatus {
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

impl std::fmt::Display for QueryReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query reservation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryReservation {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub external_ref: String,
    pub description: String,
    pub score: f64,
    pub weight: f64,
    pub rotation_days: i64,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub status: QueryReservationStatus,
}

impl QueryReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryReservationStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryReservationStatus) -> bool {
        match self.status {
            QueryReservationStatus::New => {
                return next == QueryReservationStatus::Done;
            }
            QueryReservationStatus::Queued => {
                match next {
                    QueryReservationStatus::Running | QueryReservationStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryReservationStatus::Running => {
                return next == QueryReservationStatus::Done;
            }
            QueryReservationStatus::Done => {
                match next {
                    QueryReservationStatus::Failed => {
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

    pub fn move_to(&mut self, next: QueryReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
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

    pub fn net_value(&self) -> f64 {
        ((self.weight + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryReservation {
        QueryReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            description: self.description.clone(),
            score: self.score,
            weight: self.weight,
            rotation_days: self.rotation_days,
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a query reservation in status new.
pub fn new_query_reservation(code: &str, label: &str) -> QueryReservation {
    QueryReservation {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        external_ref: "".to_string(),
        description: "".to_string(),
        score: 0.0,
        weight: 0.0,
        rotation_days: 0,
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        status: QueryReservationStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_query_reservation("code-580", "label-961");
        assert_eq!(item.status, QueryReservationStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_done() {
        let mut item = new_query_reservation("code-580", "label-961");
        if item.move_to(QueryReservationStatus::Done).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryReservationStatus::Done, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_query_reservation("code-580", "label-961");
        item.status = QueryReservationStatus::Failed;
        assert!(
            item.move_to(QueryReservationStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
