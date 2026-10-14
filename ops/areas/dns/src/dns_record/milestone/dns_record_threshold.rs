use super::*;

use acme_kit::text;

/// Where a dns record threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordThresholdStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordThresholdStatus {
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

impl std::fmt::Display for DnsRecordThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record threshold. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordThreshold {
    pub code: String,
    pub title: String,
    pub ceiling: f64,
    pub margin: f64,
    pub starts_at: i64,
    pub quantity: i64,
    pub grace_days: i64,
    pub key_id: String,
    pub port: i64,
    pub priority: i64,
    pub threshold: f64,
    pub status: DnsRecordThresholdStatus,
}

impl DnsRecordThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordThresholdStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordThresholdStatus) -> bool {
        match self.status {
            DnsRecordThresholdStatus::New => {
                return next == DnsRecordThresholdStatus::Failed;
            }
            DnsRecordThresholdStatus::Queued => {
                match next {
                    DnsRecordThresholdStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordThresholdStatus::Running => {
                return next == DnsRecordThresholdStatus::Done || next == DnsRecordThresholdStatus::Failed;
            }
            DnsRecordThresholdStatus::Done => {
                match next {
                    DnsRecordThresholdStatus::Failed => {
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

    pub fn move_to(&mut self, next: DnsRecordThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record threshold {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.grace_days as f64 + (self.starts_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordThreshold {
        DnsRecordThreshold {
            code: self.code.clone(),
            title: self.title.clone(),
            ceiling: self.ceiling,
            margin: self.margin,
            starts_at: self.starts_at,
            quantity: self.quantity,
            grace_days: self.grace_days,
            key_id: self.key_id.clone(),
            port: self.port,
            priority: self.priority,
            threshold: self.threshold,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dns record threshold in status new.
pub fn new_dns_record_threshold(code: &str, title: &str) -> DnsRecordThreshold {
    DnsRecordThreshold {
        code: code.to_string(),
        title: title.to_string(),
        ceiling: 0.0,
        margin: 0.0,
        starts_at: 0,
        quantity: 0,
        grace_days: 0,
        key_id: "".to_string(),
        port: 0,
        priority: 0,
        threshold: 0.0,
        status: DnsRecordThresholdStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_dns_record_threshold("code-747", "title-661");
        assert_eq!(item.status, DnsRecordThresholdStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_dns_record_threshold("code-747", "title-661");
        if item.move_to(DnsRecordThresholdStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DnsRecordThresholdStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_dns_record_threshold("code-747", "title-661");
        item.status = DnsRecordThresholdStatus::Failed;
        assert!(
            item.move_to(DnsRecordThresholdStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
