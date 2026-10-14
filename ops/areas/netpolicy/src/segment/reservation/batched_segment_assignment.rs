use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a batched segment assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedSegmentAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl BatchedSegmentAssignmentStatus {
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

impl std::fmt::Display for BatchedSegmentAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched segment assignment as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedSegmentAssignment {
    pub code: String,
    pub name: String,
    pub currency: String,
    pub fingerprint: String,
    pub ratio: f64,
    pub attributes: BTreeMap<String, String>,
    pub phone: String,
    pub reviewer: String,
    pub owner_id: String,
    pub status: BatchedSegmentAssignmentStatus,
}

impl BatchedSegmentAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedSegmentAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedSegmentAssignmentStatus) -> bool {
        match self.status {
            BatchedSegmentAssignmentStatus::Draft => {
                return next == BatchedSegmentAssignmentStatus::Suspended;
            }
            BatchedSegmentAssignmentStatus::Pending => {
                match next {
                    BatchedSegmentAssignmentStatus::Active | BatchedSegmentAssignmentStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSegmentAssignmentStatus::Active => {
                match next {
                    BatchedSegmentAssignmentStatus::Suspended | BatchedSegmentAssignmentStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedSegmentAssignmentStatus::Suspended => {
                return next == BatchedSegmentAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedSegmentAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched segment assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched segment assignment", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.ratio
    }

    pub fn copy(&self) -> BatchedSegmentAssignment {
        BatchedSegmentAssignment {
            code: self.code.clone(),
            name: self.name.clone(),
            currency: self.currency.clone(),
            fingerprint: self.fingerprint.clone(),
            ratio: self.ratio,
            attributes: self.attributes.clone(),
            phone: self.phone.clone(),
            reviewer: self.reviewer.clone(),
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched segment assignment in status draft.
pub fn new_batched_segment_assignment(code: &str, name: &str) -> BatchedSegmentAssignment {
    BatchedSegmentAssignment {
        code: code.to_string(),
        name: name.to_string(),
        currency: "".to_string(),
        fingerprint: "".to_string(),
        ratio: 0.0,
        attributes: BTreeMap::new(),
        phone: "".to_string(),
        reviewer: "".to_string(),
        owner_id: "".to_string(),
        status: BatchedSegmentAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_batched_segment_assignment("code-534", "name-575");
        assert_eq!(item.status, BatchedSegmentAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_batched_segment_assignment("code-534", "name-575");
        if item.move_to(BatchedSegmentAssignmentStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedSegmentAssignmentStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_batched_segment_assignment("code-534", "name-575");
        item.status = BatchedSegmentAssignmentStatus::Closed;
        assert!(
            item.move_to(BatchedSegmentAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
