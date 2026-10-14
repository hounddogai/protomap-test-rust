use super::*;

use acme_kit::text;

/// Where a tiered dns record entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredDnsRecordEntryStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl TieredDnsRecordEntryStatus {
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

impl std::fmt::Display for TieredDnsRecordEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A tiered dns record entry as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredDnsRecordEntry {
    pub code: String,
    pub name: String,
    pub timezone: String,
    pub window_days: i64,
    pub phone: String,
    pub batch_size: i64,
    pub retries: i64,
    pub reviewer: String,
    pub status: TieredDnsRecordEntryStatus,
}

impl TieredDnsRecordEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredDnsRecordEntryStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredDnsRecordEntryStatus) -> bool {
        match self.status {
            TieredDnsRecordEntryStatus::Idle => {
                match next {
                    TieredDnsRecordEntryStatus::Warming | TieredDnsRecordEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDnsRecordEntryStatus::Warming => {
                match next {
                    TieredDnsRecordEntryStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredDnsRecordEntryStatus::Ready => {
                return next == TieredDnsRecordEntryStatus::Draining || next == TieredDnsRecordEntryStatus::Stopped;
            }
            TieredDnsRecordEntryStatus::Draining => {
                return next == TieredDnsRecordEntryStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TieredDnsRecordEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered dns record entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.window_days as f64 - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TieredDnsRecordEntry {
        TieredDnsRecordEntry {
            code: self.code.clone(),
            name: self.name.clone(),
            timezone: self.timezone.clone(),
            window_days: self.window_days,
            phone: self.phone.clone(),
            batch_size: self.batch_size,
            retries: self.retries,
            reviewer: self.reviewer.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a tiered dns record entry in status idle.
pub fn new_tiered_dns_record_entry(code: &str, name: &str) -> TieredDnsRecordEntry {
    TieredDnsRecordEntry {
        code: code.to_string(),
        name: name.to_string(),
        timezone: "".to_string(),
        window_days: 0,
        phone: "".to_string(),
        batch_size: 0,
        retries: 0,
        reviewer: "".to_string(),
        status: TieredDnsRecordEntryStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_tiered_dns_record_entry("code-207", "name-532");
        assert_eq!(item.status, TieredDnsRecordEntryStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_tiered_dns_record_entry("code-207", "name-532");
        if item.move_to(TieredDnsRecordEntryStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TieredDnsRecordEntryStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_tiered_dns_record_entry("code-207", "name-532");
        item.status = TieredDnsRecordEntryStatus::Stopped;
        assert!(
            item.move_to(TieredDnsRecordEntryStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
