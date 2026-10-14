use super::*;

use acme_kit::text;

/// Where a tiered overage entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TieredOverageEntryStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl TieredOverageEntryStatus {
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

impl std::fmt::Display for TieredOverageEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a tiered overage entry and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct TieredOverageEntry {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub checksum: String,
    pub scope: String,
    pub name: String,
    pub score: f64,
    pub status: TieredOverageEntryStatus,
}

impl TieredOverageEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TieredOverageEntryStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TieredOverageEntryStatus) -> bool {
        match self.status {
            TieredOverageEntryStatus::Draft => {
                return next == TieredOverageEntryStatus::Suspended;
            }
            TieredOverageEntryStatus::Pending => {
                match next {
                    TieredOverageEntryStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TieredOverageEntryStatus::Active => {
                return next == TieredOverageEntryStatus::Closed;
            }
            TieredOverageEntryStatus::Suspended => {
                match next {
                    TieredOverageEntryStatus::Closed => {
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

    pub fn move_to(&mut self, next: TieredOverageEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "tiered overage entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "tiered overage entry", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> TieredOverageEntry {
        TieredOverageEntry {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            checksum: self.checksum.clone(),
            scope: self.scope.clone(),
            name: self.name.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a tiered overage entry in status draft.
pub fn new_tiered_overage_entry(code: &str, title: &str) -> TieredOverageEntry {
    TieredOverageEntry {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        checksum: "".to_string(),
        scope: "".to_string(),
        name: "".to_string(),
        score: 0.0,
        status: TieredOverageEntryStatus::Draft,
    }
}
