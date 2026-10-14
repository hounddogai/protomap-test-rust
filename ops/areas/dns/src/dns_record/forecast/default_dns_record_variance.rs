use super::*;

use acme_kit::text;

/// Where a default dns record variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDnsRecordVarianceStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DefaultDnsRecordVarianceStatus {
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

impl std::fmt::Display for DefaultDnsRecordVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default dns record variance, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordVariance {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub attempts: i64,
    pub unit: String,
    pub duration_minutes: i64,
    pub created_at: i64,
    pub status: DefaultDnsRecordVarianceStatus,
}

impl DefaultDnsRecordVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDnsRecordVarianceStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDnsRecordVarianceStatus) -> bool {
        match self.status {
            DefaultDnsRecordVarianceStatus::New => {
                match next {
                    DefaultDnsRecordVarianceStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDnsRecordVarianceStatus::Queued => {
                return next == DefaultDnsRecordVarianceStatus::Done || next == DefaultDnsRecordVarianceStatus::Failed;
            }
            DefaultDnsRecordVarianceStatus::Running => {
                return next == DefaultDnsRecordVarianceStatus::Done || next == DefaultDnsRecordVarianceStatus::Failed;
            }
            DefaultDnsRecordVarianceStatus::Done => {
                match next {
                    DefaultDnsRecordVarianceStatus::Failed => {
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

    pub fn move_to(&mut self, next: DefaultDnsRecordVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default dns record variance {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default dns record variance", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.duration_minutes as f64 - (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDnsRecordVariance {
        DefaultDnsRecordVariance {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            attempts: self.attempts,
            unit: self.unit.clone(),
            duration_minutes: self.duration_minutes,
            created_at: self.created_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a default dns record variance in status new.
pub fn new_default_dns_record_variance(code: &str, title: &str) -> DefaultDnsRecordVariance {
    DefaultDnsRecordVariance {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        attempts: 0,
        unit: "".to_string(),
        duration_minutes: 0,
        created_at: 0,
        status: DefaultDnsRecordVarianceStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_default_dns_record_variance("code-591", "title-861");
        assert_eq!(item.status, DefaultDnsRecordVarianceStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_failed() {
        let mut item = new_default_dns_record_variance("code-591", "title-861");
        if item.move_to(DefaultDnsRecordVarianceStatus::Failed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDnsRecordVarianceStatus::Failed, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_default_dns_record_variance("code-591", "title-861");
        item.status = DefaultDnsRecordVarianceStatus::Failed;
        assert!(
            item.move_to(DefaultDnsRecordVarianceStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
