use super::*;

use acme_kit::text;

/// Where a scheduled dns record entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledDnsRecordEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ScheduledDnsRecordEntryStatus {
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

impl std::fmt::Display for ScheduledDnsRecordEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a scheduled dns record entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledDnsRecordEntry {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub margin: f64,
    pub amount: f64,
    pub rate: f64,
    pub burst: i64,
    pub created_at: i64,
    pub checksum: String,
    pub status: ScheduledDnsRecordEntryStatus,
}

impl ScheduledDnsRecordEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledDnsRecordEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledDnsRecordEntryStatus) -> bool {
        match self.status {
            ScheduledDnsRecordEntryStatus::Idle => {
                match next {
                    ScheduledDnsRecordEntryStatus::Ready | ScheduledDnsRecordEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ScheduledDnsRecordEntryStatus::Warming => {
                return next == ScheduledDnsRecordEntryStatus::Draining;
            }
            ScheduledDnsRecordEntryStatus::Ready => {
                return next == ScheduledDnsRecordEntryStatus::Stopped;
            }
            ScheduledDnsRecordEntryStatus::Draining => {
                return next == ScheduledDnsRecordEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ScheduledDnsRecordEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled dns record entry {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.rate + (self.burst as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledDnsRecordEntry {
        ScheduledDnsRecordEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            margin: self.margin,
            amount: self.amount,
            rate: self.rate,
            burst: self.burst,
            created_at: self.created_at,
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a scheduled dns record entry in status idle.
pub fn new_scheduled_dns_record_entry(code: &str, name: &str) -> ScheduledDnsRecordEntry {
    ScheduledDnsRecordEntry {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        margin: 0.0,
        amount: 0.0,
        rate: 0.0,
        burst: 0,
        created_at: 0,
        checksum: "".to_string(),
        status: ScheduledDnsRecordEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_scheduled_dns_record_entry("code-270", "name-131");
        assert_eq!(item.status, ScheduledDnsRecordEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_scheduled_dns_record_entry("code-270", "name-131");
        if item.move_to(ScheduledDnsRecordEntryStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledDnsRecordEntryStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_scheduled_dns_record_entry("code-270", "name-131");
        item.status = ScheduledDnsRecordEntryStatus::Stopped;
        assert!(
            item.move_to(ScheduledDnsRecordEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
