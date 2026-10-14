use super::*;

use acme_kit::text;

/// Where a batched dns record notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedDnsRecordNoticeStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl BatchedDnsRecordNoticeStatus {
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

impl std::fmt::Display for BatchedDnsRecordNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched dns record notice as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedDnsRecordNotice {
    pub code: String,
    pub title: String,
    pub host: String,
    pub tags: Vec<String>,
    pub name: String,
    pub segment: String,
    pub scope: String,
    pub quantity: i64,
    pub version: i64,
    pub notes: String,
    pub duration_minutes: i64,
    pub status: BatchedDnsRecordNoticeStatus,
}

impl BatchedDnsRecordNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedDnsRecordNoticeStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedDnsRecordNoticeStatus) -> bool {
        match self.status {
            BatchedDnsRecordNoticeStatus::New => {
                return next == BatchedDnsRecordNoticeStatus::Running || next == BatchedDnsRecordNoticeStatus::Done;
            }
            BatchedDnsRecordNoticeStatus::Queued => {
                match next {
                    BatchedDnsRecordNoticeStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedDnsRecordNoticeStatus::Running => {
                return next == BatchedDnsRecordNoticeStatus::Done;
            }
            BatchedDnsRecordNoticeStatus::Done => {
                match next {
                    BatchedDnsRecordNoticeStatus::Failed => {
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

    pub fn move_to(&mut self, next: BatchedDnsRecordNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched dns record notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched dns record notice", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64 + (self.version as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedDnsRecordNotice {
        BatchedDnsRecordNotice {
            code: self.code.clone(),
            title: self.title.clone(),
            host: self.host.clone(),
            tags: self.tags.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            scope: self.scope.clone(),
            quantity: self.quantity,
            version: self.version,
            notes: self.notes.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a batched dns record notice in status new.
pub fn new_batched_dns_record_notice(code: &str, title: &str) -> BatchedDnsRecordNotice {
    BatchedDnsRecordNotice {
        code: code.to_string(),
        title: title.to_string(),
        host: "".to_string(),
        tags: Vec::new(),
        name: "".to_string(),
        segment: "".to_string(),
        scope: "".to_string(),
        quantity: 0,
        version: 0,
        notes: "".to_string(),
        duration_minutes: 0,
        status: BatchedDnsRecordNoticeStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_batched_dns_record_notice("code-532", "title-340");
        assert_eq!(item.status, BatchedDnsRecordNoticeStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_batched_dns_record_notice("code-532", "title-340");
        if item.move_to(BatchedDnsRecordNoticeStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedDnsRecordNoticeStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_batched_dns_record_notice("code-532", "title-340");
        item.status = BatchedDnsRecordNoticeStatus::Failed;
        assert!(
            item.move_to(BatchedDnsRecordNoticeStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
