use super::*;

use acme_kit::text;

/// Where a quote snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteSnapshotStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QuoteSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QuoteSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One quote snapshot, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteSnapshot {
    pub code: String,
    pub label: String,
    pub ceiling: f64,
    pub key_id: String,
    pub scope: String,
    pub cipher: String,
    pub pinned: bool,
    pub status: QuoteSnapshotStatus,
}

impl QuoteSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteSnapshotStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteSnapshotStatus) -> bool {
        match self.status {
            QuoteSnapshotStatus::Proposed => {
                return next == QuoteSnapshotStatus::Rejected || next == QuoteSnapshotStatus::Withdrawn;
            }
            QuoteSnapshotStatus::Approved => {
                match next {
                    QuoteSnapshotStatus::Rejected | QuoteSnapshotStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteSnapshotStatus::Rejected => {
                match next {
                    QuoteSnapshotStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: QuoteSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "quote snapshot", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ceiling
    }

    pub fn copy(&self) -> QuoteSnapshot {
        QuoteSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            ceiling: self.ceiling,
            key_id: self.key_id.clone(),
            scope: self.scope.clone(),
            cipher: self.cipher.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a quote snapshot in status proposed.
pub fn new_quote_snapshot(code: &str, label: &str) -> QuoteSnapshot {
    QuoteSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        ceiling: 0.0,
        key_id: "".to_string(),
        scope: "".to_string(),
        cipher: "".to_string(),
        pinned: false,
        status: QuoteSnapshotStatus::Proposed,
    }
}
