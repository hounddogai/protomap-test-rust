use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a segment assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentAssignmentStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentAssignmentStatus {
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

impl std::fmt::Display for SegmentAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment assignment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAssignment {
    pub code: String,
    pub title: String,
    pub attributes: BTreeMap<String, String>,
    pub amount: f64,
    pub external_ref: String,
    pub port: i64,
    pub phone: String,
    pub discount: f64,
    pub retries: i64,
    pub timezone: String,
    pub status: SegmentAssignmentStatus,
}

impl SegmentAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentAssignmentStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentAssignmentStatus) -> bool {
        match self.status {
            SegmentAssignmentStatus::Draft => {
                return next == SegmentAssignmentStatus::Active || next == SegmentAssignmentStatus::Closed;
            }
            SegmentAssignmentStatus::Pending => {
                return next == SegmentAssignmentStatus::Suspended || next == SegmentAssignmentStatus::Closed;
            }
            SegmentAssignmentStatus::Active => {
                return next == SegmentAssignmentStatus::Suspended;
            }
            SegmentAssignmentStatus::Suspended => {
                return next == SegmentAssignmentStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "segment assignment", self.code)
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

    pub fn effective_value(&self) -> f64 {
        ((self.port as f64 + self.discount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentAssignment {
        SegmentAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            attributes: self.attributes.clone(),
            amount: self.amount,
            external_ref: self.external_ref.clone(),
            port: self.port,
            phone: self.phone.clone(),
            discount: self.discount,
            retries: self.retries,
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment assignment in status draft.
pub fn new_segment_assignment(code: &str, title: &str) -> SegmentAssignment {
    SegmentAssignment {
        code: code.to_string(),
        title: title.to_string(),
        attributes: BTreeMap::new(),
        amount: 0.0,
        external_ref: "".to_string(),
        port: 0,
        phone: "".to_string(),
        discount: 0.0,
        retries: 0,
        timezone: "".to_string(),
        status: SegmentAssignmentStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_segment_assignment("code-111", "title-473");
        assert_eq!(item.status, SegmentAssignmentStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_segment_assignment("code-111", "title-473");
        if item.move_to(SegmentAssignmentStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentAssignmentStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_segment_assignment("code-111", "title-473");
        item.status = SegmentAssignmentStatus::Closed;
        assert!(
            item.move_to(SegmentAssignmentStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
