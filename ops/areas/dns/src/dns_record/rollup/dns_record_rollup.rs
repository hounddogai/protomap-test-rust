use super::*;

use acme_kit::text;

/// Where a dns record rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordRollupStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DnsRecordRollupStatus {
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

impl std::fmt::Display for DnsRecordRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a dns record rollup and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordRollup {
    pub code: String,
    pub name: String,
    pub ttl_seconds: i64,
    pub approved: bool,
    pub window_days: i64,
    pub version: i64,
    pub ceiling: f64,
    pub label: String,
    pub status: DnsRecordRollupStatus,
}

impl DnsRecordRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordRollupStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordRollupStatus) -> bool {
        match self.status {
            DnsRecordRollupStatus::New => {
                match next {
                    DnsRecordRollupStatus::Running | DnsRecordRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRollupStatus::Queued => {
                match next {
                    DnsRecordRollupStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRollupStatus::Running => {
                match next {
                    DnsRecordRollupStatus::Done | DnsRecordRollupStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DnsRecordRollupStatus::Done => {
                return next == DnsRecordRollupStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DnsRecordRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.window_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordRollup {
        DnsRecordRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            ttl_seconds: self.ttl_seconds,
            approved: self.approved,
            window_days: self.window_days,
            version: self.version,
            ceiling: self.ceiling,
            label: self.label.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a dns record rollup in status new.
pub fn new_dns_record_rollup(code: &str, name: &str) -> DnsRecordRollup {
    DnsRecordRollup {
        code: code.to_string(),
        name: name.to_string(),
        ttl_seconds: 0,
        approved: false,
        window_days: 0,
        version: 0,
        ceiling: 0.0,
        label: "".to_string(),
        status: DnsRecordRollupStatus::New,
    }
}
