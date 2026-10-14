use super::*;

use acme_kit::text;

/// Where a dns record schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordScheduleStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl DnsRecordScheduleStatus {
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

impl std::fmt::Display for DnsRecordScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record schedule. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordSchedule {
    pub code: String,
    pub title: String,
    pub updated_at: i64,
    pub duration_minutes: i64,
    pub tenant_id: String,
    pub grace_days: i64,
    pub minimum: f64,
    pub summary: String,
    pub status: DnsRecordScheduleStatus,
}

impl DnsRecordSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordScheduleStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordScheduleStatus) -> bool {
        match self.status {
            DnsRecordScheduleStatus::Open => {
                return next == DnsRecordScheduleStatus::InReview || next == DnsRecordScheduleStatus::Reopened;
            }
            DnsRecordScheduleStatus::InReview => {
                return next == DnsRecordScheduleStatus::Resolved || next == DnsRecordScheduleStatus::Reopened;
            }
            DnsRecordScheduleStatus::Resolved => {
                match next {
                    DnsRecordScheduleStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordScheduleStatus::Reopened => {
                match next {
                    DnsRecordScheduleStatus::Archived => {
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

    pub fn move_to(&mut self, next: DnsRecordScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        (self.minimum * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordSchedule {
        DnsRecordSchedule {
            code: self.code.clone(),
            title: self.title.clone(),
            updated_at: self.updated_at,
            duration_minutes: self.duration_minutes,
            tenant_id: self.tenant_id.clone(),
            grace_days: self.grace_days,
            minimum: self.minimum,
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dns record schedule in status open.
pub fn new_dns_record_schedule(code: &str, title: &str) -> DnsRecordSchedule {
    DnsRecordSchedule {
        code: code.to_string(),
        title: title.to_string(),
        updated_at: 0,
        duration_minutes: 0,
        tenant_id: "".to_string(),
        grace_days: 0,
        minimum: 0.0,
        summary: "".to_string(),
        status: DnsRecordScheduleStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_dns_record_schedule("code-207", "title-488");
        assert_eq!(item.status, DnsRecordScheduleStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_dns_record_schedule("code-207", "title-488");
        if item.move_to(DnsRecordScheduleStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordScheduleStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_dns_record_schedule("code-207", "title-488");
        item.status = DnsRecordScheduleStatus::Archived;
        assert!(
            item.move_to(DnsRecordScheduleStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
