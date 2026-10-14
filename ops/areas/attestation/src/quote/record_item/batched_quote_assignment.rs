use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched quote assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedQuoteAssignmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl BatchedQuoteAssignmentStatus {
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

impl std::fmt::Display for BatchedQuoteAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched quote assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedQuoteAssignment {
    pub code: String,
    pub name: String,
    pub tags: Vec<String>,
    pub attributes: BTreeMap<String, String>,
    pub ceiling: f64,
    pub batch_size: i64,
    pub status: BatchedQuoteAssignmentStatus,
}

impl BatchedQuoteAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedQuoteAssignmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedQuoteAssignmentStatus) -> bool {
        match self.status {
            BatchedQuoteAssignmentStatus::Open => {
                return next == BatchedQuoteAssignmentStatus::Reopened;
            }
            BatchedQuoteAssignmentStatus::InReview => {
                return next == BatchedQuoteAssignmentStatus::Archived;
            }
            BatchedQuoteAssignmentStatus::Resolved => {
                match next {
                    BatchedQuoteAssignmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedQuoteAssignmentStatus::Reopened => {
                return next == BatchedQuoteAssignmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedQuoteAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched quote assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
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

    pub fn weighted_total(&self) -> f64 {
        ((self.batch_size as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedQuoteAssignment {
        BatchedQuoteAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            tags: self.tags.clone(),
            attributes: self.attributes.clone(),
            ceiling: self.ceiling,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched quote assignment in status open.
pub fn new_batched_quote_assignment(code: &str, name: &str) -> BatchedQuoteAssignment {
    BatchedQuoteAssignment {
        code: code.to_string(),
        name: name.to_string(),
        tags: Vec::new(),
        attributes: BTreeMap::new(),
        ceiling: 0.0,
        batch_size: 0,
        status: BatchedQuoteAssignmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_batched_quote_assignment("code-246", "name-157");
        assert_eq!(item.status, BatchedQuoteAssignmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_batched_quote_assignment("code-246", "name-157");
        if item.move_to(BatchedQuoteAssignmentStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedQuoteAssignmentStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_batched_quote_assignment("code-246", "name-157");
        item.status = BatchedQuoteAssignmentStatus::Archived;
        assert!(
            item.move_to(BatchedQuoteAssignmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
