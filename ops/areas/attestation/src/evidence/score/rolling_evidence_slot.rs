use super::*;

use acme_kit::text;

/// Where a rolling evidence slot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEvidenceSlotStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RollingEvidenceSlotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RollingEvidenceSlotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling evidence slot as the evidence team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEvidenceSlot {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub contact_email: String,
    pub version: i64,
    pub source: String,
    pub ratio: f64,
    pub pinned: bool,
    pub status: RollingEvidenceSlotStatus,
}

impl RollingEvidenceSlot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEvidenceSlotStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEvidenceSlotStatus) -> bool {
        match self.status {
            RollingEvidenceSlotStatus::Requested => {
                match next {
                    RollingEvidenceSlotStatus::Revoked | RollingEvidenceSlotStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingEvidenceSlotStatus::Granted => {
                return next == RollingEvidenceSlotStatus::Revoked || next == RollingEvidenceSlotStatus::Expired;
            }
            RollingEvidenceSlotStatus::Revoked => {
                match next {
                    RollingEvidenceSlotStatus::Expired => {
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

    pub fn move_to(&mut self, next: RollingEvidenceSlotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling evidence slot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.batch_size as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingEvidenceSlot {
        RollingEvidenceSlot {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            contact_email: self.contact_email.clone(),
            version: self.version,
            source: self.source.clone(),
            ratio: self.ratio,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a rolling evidence slot in status requested.
pub fn new_rolling_evidence_slot(code: &str, title: &str) -> RollingEvidenceSlot {
    RollingEvidenceSlot {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        contact_email: "".to_string(),
        version: 0,
        source: "".to_string(),
        ratio: 0.0,
        pinned: false,
        status: RollingEvidenceSlotStatus::Requested,
    }
}
