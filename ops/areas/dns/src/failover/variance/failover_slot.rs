use super::*;

use acme_kit::text;

/// Where a failover slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FailoverSlotStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FailoverSlotStatus {
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

impl std::fmt::Display for FailoverSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A failover slot. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FailoverSlot {
    pub code: String,
    pub label: String,
    pub cipher: String,
    pub source: String,
    pub priority: i64,
    pub issuer: String,
    pub external_ref: String,
    pub margin: f64,
    pub amount: f64,
    pub status: FailoverSlotStatus,
}

impl FailoverSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FailoverSlotStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FailoverSlotStatus) -> bool {
        match self.status {
            FailoverSlotStatus::New => {
                match next {
                    FailoverSlotStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverSlotStatus::Queued => {
                match next {
                    FailoverSlotStatus::Running | FailoverSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverSlotStatus::Running => {
                match next {
                    FailoverSlotStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FailoverSlotStatus::Done => {
                return next == FailoverSlotStatus::Failed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FailoverSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "failover slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "failover slot", self.code)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.amount - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FailoverSlot {
        FailoverSlot {
            code: self.code.clone(),
            label: self.label.clone(),
            cipher: self.cipher.clone(),
            source: self.source.clone(),
            priority: self.priority,
            issuer: self.issuer.clone(),
            external_ref: self.external_ref.clone(),
            margin: self.margin,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a failover slot in status new.
pub fn new_failover_slot(code: &str, label: &str) -> FailoverSlot {
    FailoverSlot {
        code: code.to_string(),
        label: label.to_string(),
        cipher: "".to_string(),
        source: "".to_string(),
        priority: 0,
        issuer: "".to_string(),
        external_ref: "".to_string(),
        margin: 0.0,
        amount: 0.0,
        status: FailoverSlotStatus::New,
    }
}
