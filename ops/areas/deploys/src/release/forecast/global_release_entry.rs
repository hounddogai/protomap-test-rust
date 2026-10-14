use super::*;

use acme_kit::text;

/// Where a global release entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalReleaseEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl GlobalReleaseEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for GlobalReleaseEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a global release entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalReleaseEntry {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub floor_value: f64,
    pub segment: String,
    pub summary: String,
    pub status: GlobalReleaseEntryStatus,
}

impl GlobalReleaseEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalReleaseEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalReleaseEntryStatus) -> bool {
        match self.status {
            GlobalReleaseEntryStatus::Requested => {
                return next == GlobalReleaseEntryStatus::Granted || next == GlobalReleaseEntryStatus::Revoked;
            }
            GlobalReleaseEntryStatus::Granted => {
                return next == GlobalReleaseEntryStatus::Revoked;
            }
            GlobalReleaseEntryStatus::Revoked => {
                return next == GlobalReleaseEntryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalReleaseEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global release entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_load(&self) -> f64 {
        ((self.floor_value + self.score) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalReleaseEntry {
        GlobalReleaseEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            floor_value: self.floor_value,
            segment: self.segment.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a global release entry in status requested.
pub fn new_global_release_entry(code: &str, label: &str) -> GlobalReleaseEntry {
    GlobalReleaseEntry {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        floor_value: 0.0,
        segment: "".to_string(),
        summary: "".to_string(),
        status: GlobalReleaseEntryStatus::Requested,
    }
}
