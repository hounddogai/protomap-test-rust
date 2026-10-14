use super::*;

use acme_kit::text;

/// Where a dns record exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordExceptionStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl DnsRecordExceptionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for DnsRecordExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordException {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub segment: String,
    pub revision: i64,
    pub expires_at: i64,
    pub channel: String,
    pub rotation_days: i64,
    pub ends_at: i64,
    pub status: DnsRecordExceptionStatus,
}

impl DnsRecordException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordExceptionStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordExceptionStatus) -> bool {
        match self.status {
            DnsRecordExceptionStatus::Proposed => {
                match next {
                    DnsRecordExceptionStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordExceptionStatus::Approved => {
                return next == DnsRecordExceptionStatus::Rejected;
            }
            DnsRecordExceptionStatus::Rejected => {
                return next == DnsRecordExceptionStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dns record exception", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.rotation_days as f64 - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordException {
        DnsRecordException {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            segment: self.segment.clone(),
            revision: self.revision,
            expires_at: self.expires_at,
            channel: self.channel.clone(),
            rotation_days: self.rotation_days,
            ends_at: self.ends_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a dns record exception in status proposed.
pub fn new_dns_record_exception(code: &str, title: &str) -> DnsRecordException {
    DnsRecordException {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        segment: "".to_string(),
        revision: 0,
        expires_at: 0,
        channel: "".to_string(),
        rotation_days: 0,
        ends_at: 0,
        status: DnsRecordExceptionStatus::Proposed,
    }
}
