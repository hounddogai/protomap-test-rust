use super::*;

use acme_kit::text;

/// Where a evidence entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EvidenceEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl EvidenceEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for EvidenceEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a evidence entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceEntry {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub amount: f64,
    pub owner_id: String,
    pub status: EvidenceEntryStatus,
}

impl EvidenceEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EvidenceEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EvidenceEntryStatus) -> bool {
        match self.status {
            EvidenceEntryStatus::Draft => {
                return next == EvidenceEntryStatus::Pending;
            }
            EvidenceEntryStatus::Pending => {
                match next {
                    EvidenceEntryStatus::Suspended | EvidenceEntryStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EvidenceEntryStatus::Active => {
                return next == EvidenceEntryStatus::Suspended;
            }
            EvidenceEntryStatus::Suspended => {
                return next == EvidenceEntryStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EvidenceEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "evidence entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "evidence entry", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> EvidenceEntry {
        EvidenceEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            amount: self.amount,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a evidence entry in status draft.
pub fn new_evidence_entry(code: &str, title: &str) -> EvidenceEntry {
    EvidenceEntry {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        amount: 0.0,
        owner_id: "".to_string(),
        status: EvidenceEntryStatus::Draft,
    }
}
