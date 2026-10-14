use super::*;

use acme_kit::text;

/// Where a global dns record cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalDnsRecordCycleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalDnsRecordCycleStatus {
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

impl std::fmt::Display for GlobalDnsRecordCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global dns record cycle, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalDnsRecordCycle {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub updated_at: i64,
    pub source: String,
    pub amount: f64,
    pub status: GlobalDnsRecordCycleStatus,
}

impl GlobalDnsRecordCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalDnsRecordCycleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalDnsRecordCycleStatus) -> bool {
        match self.status {
            GlobalDnsRecordCycleStatus::Idle => {
                return next == GlobalDnsRecordCycleStatus::Warming;
            }
            GlobalDnsRecordCycleStatus::Warming => {
                return next == GlobalDnsRecordCycleStatus::Ready || next == GlobalDnsRecordCycleStatus::Draining;
            }
            GlobalDnsRecordCycleStatus::Ready => {
                return next == GlobalDnsRecordCycleStatus::Draining;
            }
            GlobalDnsRecordCycleStatus::Draining => {
                return next == GlobalDnsRecordCycleStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalDnsRecordCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global dns record cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "global dns record cycle", self.code)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.updated_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalDnsRecordCycle {
        GlobalDnsRecordCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            updated_at: self.updated_at,
            source: self.source.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a global dns record cycle in status idle.
pub fn new_global_dns_record_cycle(code: &str, label: &str) -> GlobalDnsRecordCycle {
    GlobalDnsRecordCycle {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        updated_at: 0,
        source: "".to_string(),
        amount: 0.0,
        status: GlobalDnsRecordCycleStatus::Idle,
    }
}
