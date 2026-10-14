use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a usage segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl UsageSegmentStatus {
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

impl std::fmt::Display for UsageSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One usage segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageSegment {
    pub code: String,
    pub label: String,
    pub approved: bool,
    pub attempts: i64,
    pub margin: f64,
    pub attributes: BTreeMap<String, String>,
    pub country: String,
    pub region: String,
    pub maximum: f64,
    pub issuer: String,
    pub tenant_id: String,
    pub status: UsageSegmentStatus,
}

impl UsageSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageSegmentStatus) -> bool {
        match self.status {
            UsageSegmentStatus::Open => {
                match next {
                    UsageSegmentStatus::InReview | UsageSegmentStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageSegmentStatus::InReview => {
                match next {
                    UsageSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageSegmentStatus::Resolved => {
                return next == UsageSegmentStatus::Archived;
            }
            UsageSegmentStatus::Reopened => {
                return next == UsageSegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage segment {} cannot move from {} to {}",
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

    pub fn net_volume(&self) -> f64 {
        ((self.attempts as f64 - self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageSegment {
        UsageSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            approved: self.approved,
            attempts: self.attempts,
            margin: self.margin,
            attributes: self.attributes.clone(),
            country: self.country.clone(),
            region: self.region.clone(),
            maximum: self.maximum,
            issuer: self.issuer.clone(),
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a usage segment in status open.
pub fn new_usage_segment(code: &str, label: &str) -> UsageSegment {
    UsageSegment {
        code: code.to_string(),
        label: label.to_string(),
        approved: false,
        attempts: 0,
        margin: 0.0,
        attributes: BTreeMap::new(),
        country: "".to_string(),
        region: "".to_string(),
        maximum: 0.0,
        issuer: "".to_string(),
        tenant_id: "".to_string(),
        status: UsageSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_usage_segment("code-615", "label-368");
        assert_eq!(item.status, UsageSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_usage_segment("code-615", "label-368");
        if item.move_to(UsageSegmentStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, UsageSegmentStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_usage_segment("code-615", "label-368");
        item.status = UsageSegmentStatus::Archived;
        assert!(item.move_to(UsageSegmentStatus::Open).is_err(), "expected move to fail");
    }
}
