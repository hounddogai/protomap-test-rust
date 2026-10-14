use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a policy segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PolicySegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PolicySegmentStatus {
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

impl std::fmt::Display for PolicySegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One policy segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PolicySegment {
    pub code: String,
    pub label: String,
    pub cost: f64,
    pub unit: String,
    pub attributes: BTreeMap<String, String>,
    pub status: PolicySegmentStatus,
}

impl PolicySegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PolicySegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PolicySegmentStatus) -> bool {
        match self.status {
            PolicySegmentStatus::Open => {
                match next {
                    PolicySegmentStatus::Resolved | PolicySegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PolicySegmentStatus::InReview => {
                return next == PolicySegmentStatus::Reopened;
            }
            PolicySegmentStatus::Resolved => {
                return next == PolicySegmentStatus::Reopened;
            }
            PolicySegmentStatus::Reopened => {
                return next == PolicySegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PolicySegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "policy segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
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
        self.cost
    }

    pub fn copy(&self) -> PolicySegment {
        PolicySegment {
            code: self.code.clone(),
            label: self.label.clone(),
            cost: self.cost,
            unit: self.unit.clone(),
            attributes: self.attributes.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a policy segment in status open.
pub fn new_policy_segment(code: &str, label: &str) -> PolicySegment {
    PolicySegment {
        code: code.to_string(),
        label: label.to_string(),
        cost: 0.0,
        unit: "".to_string(),
        attributes: BTreeMap::new(),
        status: PolicySegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_policy_segment("code-740", "label-231");
        assert_eq!(item.status, PolicySegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_policy_segment("code-740", "label-231");
        if item.move_to(PolicySegmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PolicySegmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_policy_segment("code-740", "label-231");
        item.status = PolicySegmentStatus::Archived;
        assert!(
            item.move_to(PolicySegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
