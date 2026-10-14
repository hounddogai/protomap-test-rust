use super::*;

use acme_kit::text;

/// Where a evidence slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceSlotStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EvidenceSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for EvidenceSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence slot and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceSlot {
    pub code: String,
    pub title: String,
    pub window_days: i64,
    pub tenant_id: String,
    pub owner_id: String,
    pub expires_at: i64,
    pub checksum: String,
    pub grace_days: i64,
    pub label: String,
    pub retries: i64,
    pub amount: f64,
    pub status: EvidenceSlotStatus,
}

impl EvidenceSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceSlotStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceSlotStatus) -> bool {
        match self.status {
            EvidenceSlotStatus::Planned => {
                match next {
                    EvidenceSlotStatus::Scheduled | EvidenceSlotStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceSlotStatus::Scheduled => {
                match next {
                    EvidenceSlotStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceSlotStatus::InProgress => {
                return next == EvidenceSlotStatus::Completed || next == EvidenceSlotStatus::Cancelled;
            }
            EvidenceSlotStatus::Completed => {
                match next {
                    EvidenceSlotStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EvidenceSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence slot {} cannot move from {} to {}",
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
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.window_days as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EvidenceSlot {
        EvidenceSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            window_days: self.window_days,
            tenant_id: self.tenant_id.clone(),
            owner_id: self.owner_id.clone(),
            expires_at: self.expires_at,
            checksum: self.checksum.clone(),
            grace_days: self.grace_days,
            label: self.label.clone(),
            retries: self.retries,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a evidence slot in status planned.
pub fn new_evidence_slot(code: &str, title: &str) -> EvidenceSlot {
    EvidenceSlot {
        code: code.to_string(),
        title: title.to_string(),
        window_days: 0,
        tenant_id: "".to_string(),
        owner_id: "".to_string(),
        expires_at: 0,
        checksum: "".to_string(),
        grace_days: 0,
        label: "".to_string(),
        retries: 0,
        amount: 0.0,
        status: EvidenceSlotStatus::Planned,
    }
}
