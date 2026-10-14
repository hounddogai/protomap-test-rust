use super::*;

use acme_kit::text;

/// Where a scheduled dns record segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDnsRecordSegmentStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledDnsRecordSegmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledDnsRecordSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A scheduled dns record segment as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDnsRecordSegment {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub reviewer: String,
    pub ends_at: i64,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub status: ScheduledDnsRecordSegmentStatus,
}

impl ScheduledDnsRecordSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDnsRecordSegmentStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDnsRecordSegmentStatus) -> bool {
        match self.status {
            ScheduledDnsRecordSegmentStatus::Proposed => {
                return next == ScheduledDnsRecordSegmentStatus::Approved
                    || next == ScheduledDnsRecordSegmentStatus::Rejected;
            }
            ScheduledDnsRecordSegmentStatus::Approved => {
                return next == ScheduledDnsRecordSegmentStatus::Rejected
                    || next == ScheduledDnsRecordSegmentStatus::Withdrawn;
            }
            ScheduledDnsRecordSegmentStatus::Rejected => {
                match next {
                    ScheduledDnsRecordSegmentStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledDnsRecordSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled dns record segment {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDnsRecordSegment {
        ScheduledDnsRecordSegment {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            reviewer: self.reviewer.clone(),
            ends_at: self.ends_at,
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a scheduled dns record segment in status proposed.
pub fn new_scheduled_dns_record_segment(code: &str, label: &str) -> ScheduledDnsRecordSegment {
    ScheduledDnsRecordSegment {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        reviewer: "".to_string(),
        ends_at: 0,
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        status: ScheduledDnsRecordSegmentStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_dns_record_segment("code-757", "label-970");
        assert_eq!(item.status, ScheduledDnsRecordSegmentStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_dns_record_segment("code-757", "label-970");
        if item.move_to(ScheduledDnsRecordSegmentStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDnsRecordSegmentStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_dns_record_segment("code-757", "label-970");
        item.status = ScheduledDnsRecordSegmentStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledDnsRecordSegmentStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
