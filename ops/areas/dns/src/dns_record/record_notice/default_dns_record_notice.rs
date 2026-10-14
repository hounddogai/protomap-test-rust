use super::*;

use acme_kit::text;

/// Where a default dns record notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDnsRecordNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultDnsRecordNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for DefaultDnsRecordNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default dns record notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordNotice {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub country: String,
    pub capacity: i64,
    pub attempts: i64,
    pub amount: f64,
    pub sequence: i64,
    pub threshold: f64,
    pub segment: String,
    pub status: DefaultDnsRecordNoticeStatus,
}

impl DefaultDnsRecordNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDnsRecordNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDnsRecordNoticeStatus) -> bool {
        match self.status {
            DefaultDnsRecordNoticeStatus::New => {
                return next == DefaultDnsRecordNoticeStatus::Queued;
            }
            DefaultDnsRecordNoticeStatus::Queued => {
                return next == DefaultDnsRecordNoticeStatus::Failed;
            }
            DefaultDnsRecordNoticeStatus::Running => {
                return next == DefaultDnsRecordNoticeStatus::Done;
            }
            DefaultDnsRecordNoticeStatus::Done => {
                return next == DefaultDnsRecordNoticeStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultDnsRecordNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default dns record notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.threshold + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDnsRecordNotice {
        DefaultDnsRecordNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            country: self.country.clone(),
            capacity: self.capacity,
            attempts: self.attempts,
            amount: self.amount,
            sequence: self.sequence,
            threshold: self.threshold,
            segment: self.segment.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a default dns record notice in status new.
pub fn new_default_dns_record_notice(code: &str, title: &str) -> DefaultDnsRecordNotice {
    DefaultDnsRecordNotice {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        country: "".to_string(),
        capacity: 0,
        attempts: 0,
        amount: 0.0,
        sequence: 0,
        threshold: 0.0,
        segment: "".to_string(),
        status: DefaultDnsRecordNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_dns_record_notice("code-183", "title-396");
        assert_eq!(item.status, DefaultDnsRecordNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_default_dns_record_notice("code-183", "title-396");
        if item.move_to(DefaultDnsRecordNoticeStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDnsRecordNoticeStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_dns_record_notice("code-183", "title-396");
        item.status = DefaultDnsRecordNoticeStatus::Failed;
        assert!(
            item.move_to(DefaultDnsRecordNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
