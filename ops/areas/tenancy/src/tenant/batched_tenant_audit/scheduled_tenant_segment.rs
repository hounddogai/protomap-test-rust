use super::*;

use acme_kit::text;

/// Where a scheduled tenant segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledTenantSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ScheduledTenantSegmentStatus {
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

impl std::fmt::Display for ScheduledTenantSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled tenant segment as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledTenantSegment {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub cost: f64,
    pub sequence: i64,
    pub status: ScheduledTenantSegmentStatus,
}

impl ScheduledTenantSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledTenantSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledTenantSegmentStatus) -> bool {
        match self.status {
            ScheduledTenantSegmentStatus::Open => {
                return next == ScheduledTenantSegmentStatus::Resolved;
            }
            ScheduledTenantSegmentStatus::InReview => {
                return next == ScheduledTenantSegmentStatus::Reopened;
            }
            ScheduledTenantSegmentStatus::Resolved => {
                match next {
                    ScheduledTenantSegmentStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledTenantSegmentStatus::Reopened => {
                match next {
                    ScheduledTenantSegmentStatus::Archived => {
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

    pub fn move_to(&mut self, next: ScheduledTenantSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled tenant segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "scheduled tenant segment", self.code)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.sequence as f64) * self.cost * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledTenantSegment {
        ScheduledTenantSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            cost: self.cost,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a scheduled tenant segment in status open.
pub fn new_scheduled_tenant_segment(code: &str, title: &str) -> ScheduledTenantSegment {
    ScheduledTenantSegment {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        cost: 0.0,
        sequence: 0,
        status: ScheduledTenantSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_scheduled_tenant_segment("code-660", "title-743");
        assert_eq!(item.status, ScheduledTenantSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_scheduled_tenant_segment("code-660", "title-743");
        if item.move_to(ScheduledTenantSegmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledTenantSegmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_scheduled_tenant_segment("code-660", "title-743");
        item.status = ScheduledTenantSegmentStatus::Archived;
        assert!(
            item.move_to(ScheduledTenantSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
