use super::*;

use acme_kit::text;

/// Where a dns record cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DnsRecordCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DnsRecordCycleStatus {
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

impl std::fmt::Display for DnsRecordCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A dns record cycle as the dns record team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DnsRecordCycle {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub key_id: String,
    pub minimum: f64,
    pub duration_minutes: i64,
    pub budget: f64,
    pub status: DnsRecordCycleStatus,
}

impl DnsRecordCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DnsRecordCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DnsRecordCycleStatus) -> bool {
        match self.status {
            DnsRecordCycleStatus::Idle => {
                return next == DnsRecordCycleStatus::Draining;
            }
            DnsRecordCycleStatus::Warming => {
                return next == DnsRecordCycleStatus::Ready;
            }
            DnsRecordCycleStatus::Ready => {
                return next == DnsRecordCycleStatus::Draining || next == DnsRecordCycleStatus::Stopped;
            }
            DnsRecordCycleStatus::Draining => {
                match next {
                    DnsRecordCycleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: DnsRecordCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "dns record cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "dns record cycle", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.budget - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DnsRecordCycle {
        DnsRecordCycle {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            key_id: self.key_id.clone(),
            minimum: self.minimum,
            duration_minutes: self.duration_minutes,
            budget: self.budget,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a dns record cycle in status idle.
pub fn new_dns_record_cycle(code: &str, title: &str) -> DnsRecordCycle {
    DnsRecordCycle {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        key_id: "".to_string(),
        minimum: 0.0,
        duration_minutes: 0,
        budget: 0.0,
        status: DnsRecordCycleStatus::Idle,
    }
}
