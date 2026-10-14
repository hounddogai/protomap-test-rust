use super::*;

use acme_kit::text;

/// Where a dns record request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordRequestStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordRequestStatus {
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

impl std::fmt::Display for DnsRecordRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record request and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRequest {
    pub code: String,
    pub label: String,
    pub channel: String,
    pub active: bool,
    pub expires_at: i64,
    pub threshold: f64,
    pub duration_minutes: i64,
    pub region: String,
    pub attempts: i64,
    pub status: DnsRecordRequestStatus,
}

impl DnsRecordRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordRequestStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordRequestStatus) -> bool {
        match self.status {
            DnsRecordRequestStatus::New => {
                match next {
                    DnsRecordRequestStatus::Queued | DnsRecordRequestStatus::Running => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRequestStatus::Queued => {
                match next {
                    DnsRecordRequestStatus::Done | DnsRecordRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRequestStatus::Running => {
                match next {
                    DnsRecordRequestStatus::Done | DnsRecordRequestStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRequestStatus::Done => {
                return next == DnsRecordRequestStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.attempts as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordRequest {
        DnsRecordRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            channel: self.channel.clone(),
            active: self.active,
            expires_at: self.expires_at,
            threshold: self.threshold,
            duration_minutes: self.duration_minutes,
            region: self.region.clone(),
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a dns record request in status new.
pub fn new_dns_record_request(code: &str, label: &str) -> DnsRecordRequest {
    DnsRecordRequest {
        code: code.to_string(),
        label: label.to_string(),
        channel: "".to_string(),
        active: false,
        expires_at: 0,
        threshold: 0.0,
        duration_minutes: 0,
        region: "".to_string(),
        attempts: 0,
        status: DnsRecordRequestStatus::New,
    }
}
