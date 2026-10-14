use super::*;

use acme_kit::text;

/// Where a rolling audit event segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingAuditEventSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl RollingAuditEventSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for RollingAuditEventSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One rolling audit event segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingAuditEventSegment {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub phone: String,
    pub floor_value: f64,
    pub summary: String,
    pub ratio: f64,
    pub ends_at: i64,
    pub pinned: bool,
    pub batch_size: i64,
    pub status: RollingAuditEventSegmentStatus,
}

impl RollingAuditEventSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingAuditEventSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingAuditEventSegmentStatus) -> bool {
        match self.status {
            RollingAuditEventSegmentStatus::Proposed => {
                match next {
                    RollingAuditEventSegmentStatus::Rejected | RollingAuditEventSegmentStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingAuditEventSegmentStatus::Approved => {
                return next == RollingAuditEventSegmentStatus::Rejected
                    || next == RollingAuditEventSegmentStatus::Withdrawn;
            }
            RollingAuditEventSegmentStatus::Rejected => {
                return next == RollingAuditEventSegmentStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingAuditEventSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling audit event segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling audit event segment", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.floor_value * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingAuditEventSegment {
        RollingAuditEventSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            phone: self.phone.clone(),
            floor_value: self.floor_value,
            summary: self.summary.clone(),
            ratio: self.ratio,
            ends_at: self.ends_at,
            pinned: self.pinned,
            batch_size: self.batch_size,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a rolling audit event segment in status proposed.
pub fn new_rolling_audit_event_segment(code: &str, label: &str) -> RollingAuditEventSegment {
    RollingAuditEventSegment {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        phone: "".to_string(),
        floor_value: 0.0,
        summary: "".to_string(),
        ratio: 0.0,
        ends_at: 0,
        pinned: false,
        batch_size: 0,
        status: RollingAuditEventSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_rolling_audit_event_segment("code-59", "label-386");
        assert_eq!(item.status, RollingAuditEventSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_rolling_audit_event_segment("code-59", "label-386");
        if item.move_to(RollingAuditEventSegmentStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingAuditEventSegmentStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_rolling_audit_event_segment("code-59", "label-386");
        item.status = RollingAuditEventSegmentStatus::Withdrawn;
        assert!(
            item.move_to(RollingAuditEventSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
