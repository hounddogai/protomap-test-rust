use super::*;

use acme_kit::text;

/// Where a segment audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentAuditStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentAuditStatus {
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

impl std::fmt::Display for SegmentAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment audit as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentAudit {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub reviewer: String,
    pub margin: f64,
    pub starts_at: i64,
    pub duration_minutes: i64,
    pub pinned: bool,
    pub status: SegmentAuditStatus,
}

impl SegmentAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentAuditStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentAuditStatus) -> bool {
        match self.status {
            SegmentAuditStatus::Open => {
                match next {
                    SegmentAuditStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentAuditStatus::InReview => {
                return next == SegmentAuditStatus::Resolved || next == SegmentAuditStatus::Archived;
            }
            SegmentAuditStatus::Resolved => {
                return next == SegmentAuditStatus::Reopened;
            }
            SegmentAuditStatus::Reopened => {
                return next == SegmentAuditStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_load(&self) -> f64 {
        ((self.margin + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentAudit {
        SegmentAudit {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            reviewer: self.reviewer.clone(),
            margin: self.margin,
            starts_at: self.starts_at,
            duration_minutes: self.duration_minutes,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a segment audit in status open.
pub fn new_segment_audit(code: &str, name: &str) -> SegmentAudit {
    SegmentAudit {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        reviewer: "".to_string(),
        margin: 0.0,
        starts_at: 0,
        duration_minutes: 0,
        pinned: false,
        status: SegmentAuditStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_segment_audit("code-130", "name-596");
        assert_eq!(item.status, SegmentAuditStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_segment_audit("code-130", "name-596");
        if item.move_to(SegmentAuditStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentAuditStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_segment_audit("code-130", "name-596");
        item.status = SegmentAuditStatus::Archived;
        assert!(item.move_to(SegmentAuditStatus::Open).is_err(), "expected move to fail");
    }
}
