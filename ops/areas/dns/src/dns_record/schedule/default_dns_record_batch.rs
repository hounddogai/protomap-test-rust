use super::*;

use acme_kit::text;

/// Where a default dns record batch is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultDnsRecordBatchStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultDnsRecordBatchStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for DefaultDnsRecordBatchStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default dns record batch, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultDnsRecordBatch {
    pub code: String,
    pub label: String,
    pub description: String,
    pub owner_id: String,
    pub source: String,
    pub rate: f64,
    pub quota_units: i64,
    pub name: String,
    pub reviewer: String,
    pub summary: String,
    pub status: DefaultDnsRecordBatchStatus,
}

impl DefaultDnsRecordBatch {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultDnsRecordBatchStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultDnsRecordBatchStatus) -> bool {
        match self.status {
            DefaultDnsRecordBatchStatus::Idle => {
                return next == DefaultDnsRecordBatchStatus::Draining;
            }
            DefaultDnsRecordBatchStatus::Warming => {
                return next == DefaultDnsRecordBatchStatus::Draining;
            }
            DefaultDnsRecordBatchStatus::Ready => {
                match next {
                    DefaultDnsRecordBatchStatus::Draining | DefaultDnsRecordBatchStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultDnsRecordBatchStatus::Draining => {
                match next {
                    DefaultDnsRecordBatchStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DefaultDnsRecordBatchStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default dns record batch {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "default dns record batch", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quota_units as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultDnsRecordBatch {
        DefaultDnsRecordBatch {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            owner_id: self.owner_id.clone(),
            source: self.source.clone(),
            rate: self.rate,
            quota_units: self.quota_units,
            name: self.name.clone(),
            reviewer: self.reviewer.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a default dns record batch in status idle.
pub fn new_default_dns_record_batch(code: &str, label: &str) -> DefaultDnsRecordBatch {
    DefaultDnsRecordBatch {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        owner_id: "".to_string(),
        source: "".to_string(),
        rate: 0.0,
        quota_units: 0,
        name: "".to_string(),
        reviewer: "".to_string(),
        summary: "".to_string(),
        status: DefaultDnsRecordBatchStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_dns_record_batch("code-812", "label-626");
        assert_eq!(item.status, DefaultDnsRecordBatchStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_default_dns_record_batch("code-812", "label-626");
        if item.move_to(DefaultDnsRecordBatchStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultDnsRecordBatchStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_dns_record_batch("code-812", "label-626");
        item.status = DefaultDnsRecordBatchStatus::Stopped;
        assert!(
            item.move_to(DefaultDnsRecordBatchStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
