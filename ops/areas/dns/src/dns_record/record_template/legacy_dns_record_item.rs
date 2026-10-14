use super::*;

use acme_kit::text;

/// Where a legacy dns record item is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyDnsRecordItemStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LegacyDnsRecordItemStatus {
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

impl std::fmt::Display for LegacyDnsRecordItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a legacy dns record item and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyDnsRecordItem {
    pub code: String,
    pub name: String,
    pub batch_size: i64,
    pub country: String,
    pub retries: i64,
    pub quota_units: i64,
    pub margin: f64,
    pub minimum: f64,
    pub attempts: i64,
    pub cipher: String,
    pub status: LegacyDnsRecordItemStatus,
}

impl LegacyDnsRecordItem {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyDnsRecordItemStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyDnsRecordItemStatus) -> bool {
        match self.status {
            LegacyDnsRecordItemStatus::Idle => {
                return next == LegacyDnsRecordItemStatus::Stopped;
            }
            LegacyDnsRecordItemStatus::Warming => {
                return next == LegacyDnsRecordItemStatus::Ready || next == LegacyDnsRecordItemStatus::Stopped;
            }
            LegacyDnsRecordItemStatus::Ready => {
                return next == LegacyDnsRecordItemStatus::Stopped;
            }
            LegacyDnsRecordItemStatus::Draining => {
                return next == LegacyDnsRecordItemStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyDnsRecordItemStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy dns record item {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.batch_size as f64 + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyDnsRecordItem {
        LegacyDnsRecordItem {
            code: self.code.clone(),
            name: self.name.clone(),
            batch_size: self.batch_size,
            country: self.country.clone(),
            retries: self.retries,
            quota_units: self.quota_units,
            margin: self.margin,
            minimum: self.minimum,
            attempts: self.attempts,
            cipher: self.cipher.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a legacy dns record item in status idle.
pub fn new_legacy_dns_record_item(code: &str, name: &str) -> LegacyDnsRecordItem {
    LegacyDnsRecordItem {
        code: code.to_string(),
        name: name.to_string(),
        batch_size: 0,
        country: "".to_string(),
        retries: 0,
        quota_units: 0,
        margin: 0.0,
        minimum: 0.0,
        attempts: 0,
        cipher: "".to_string(),
        status: LegacyDnsRecordItemStatus::Idle,
    }
}
