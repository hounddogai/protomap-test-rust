use super::*;

use acme_kit::text;

/// Where a pending silence notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingSilenceNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingSilenceNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingSilenceNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending silence notice and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingSilenceNotice {
    pub code: String,
    pub name: String,
    pub sequence: i64,
    pub source: String,
    pub locale: String,
    pub rate: f64,
    pub key_id: String,
    pub currency: String,
    pub tags: Vec<String>,
    pub status: PendingSilenceNoticeStatus,
}

impl PendingSilenceNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingSilenceNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingSilenceNoticeStatus) -> bool {
        match self.status {
            PendingSilenceNoticeStatus::Proposed => {
                match next {
                    PendingSilenceNoticeStatus::Approved | PendingSilenceNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingSilenceNoticeStatus::Approved => {
                return next == PendingSilenceNoticeStatus::Withdrawn;
            }
            PendingSilenceNoticeStatus::Rejected => {
                match next {
                    PendingSilenceNoticeStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingSilenceNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending silence notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.sequence as f64 - self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingSilenceNotice {
        PendingSilenceNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            sequence: self.sequence,
            source: self.source.clone(),
            locale: self.locale.clone(),
            rate: self.rate,
            key_id: self.key_id.clone(),
            currency: self.currency.clone(),
            tags: self.tags.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a pending silence notice in status proposed.
pub fn new_pending_silence_notice(code: &str, name: &str) -> PendingSilenceNotice {
    PendingSilenceNotice {
        code: code.to_string(),
        name: name.to_string(),
        sequence: 0,
        source: "".to_string(),
        locale: "".to_string(),
        rate: 0.0,
        key_id: "".to_string(),
        currency: "".to_string(),
        tags: Vec::new(),
        status: PendingSilenceNoticeStatus::Proposed,
    }
}
