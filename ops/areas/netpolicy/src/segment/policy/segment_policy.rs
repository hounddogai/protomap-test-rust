use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a segment policy is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentPolicyStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentPolicyStatus {
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

impl std::fmt::Display for SegmentPolicyStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a segment policy and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentPolicy {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub threshold: f64,
    pub weight: f64,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub unit: String,
    pub status: SegmentPolicyStatus,
}

impl SegmentPolicy {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentPolicyStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentPolicyStatus) -> bool {
        match self.status {
            SegmentPolicyStatus::Open => {
                return next == SegmentPolicyStatus::InReview || next == SegmentPolicyStatus::Archived;
            }
            SegmentPolicyStatus::InReview => {
                match next {
                    SegmentPolicyStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentPolicyStatus::Resolved => {
                match next {
                    SegmentPolicyStatus::Reopened | SegmentPolicyStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentPolicyStatus::Reopened => {
                return next == SegmentPolicyStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentPolicyStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment policy {} cannot move from {} to {}",
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

    pub fn projected_value(&self) -> f64 {
        ((self.threshold - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentPolicy {
        SegmentPolicy {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            threshold: self.threshold,
            weight: self.weight,
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            unit: self.unit.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a segment policy in status open.
pub fn new_segment_policy(code: &str, label: &str) -> SegmentPolicy {
    SegmentPolicy {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        threshold: 0.0,
        weight: 0.0,
        contact_email: "".to_string(),
        ttl_seconds: 0,
        unit: "".to_string(),
        status: SegmentPolicyStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_segment_policy("code-56", "label-773");
        assert_eq!(item.status, SegmentPolicyStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_segment_policy("code-56", "label-773");
        if item.move_to(SegmentPolicyStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentPolicyStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_segment_policy("code-56", "label-773");
        item.status = SegmentPolicyStatus::Archived;
        assert!(
            item.move_to(SegmentPolicyStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
