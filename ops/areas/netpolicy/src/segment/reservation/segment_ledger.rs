use super::*;

use acme_kit::text;

/// Where a segment ledger is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentLedgerStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentLedgerStatus {
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

impl std::fmt::Display for SegmentLedgerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment ledger, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentLedger {
    pub code: String,
    pub title: String,
    pub tags: Vec<String>,
    pub retries: i64,
    pub name: String,
    pub summary: String,
    pub status: SegmentLedgerStatus,
}

impl SegmentLedger {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentLedgerStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentLedgerStatus) -> bool {
        match self.status {
            SegmentLedgerStatus::Draft => {
                match next {
                    SegmentLedgerStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentLedgerStatus::Pending => {
                match next {
                    SegmentLedgerStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentLedgerStatus::Active => {
                return next == SegmentLedgerStatus::Suspended;
            }
            SegmentLedgerStatus::Suspended => {
                return next == SegmentLedgerStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentLedgerStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment ledger {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn measured_value(&self) -> f64 {
        self.retries as f64
    }

    pub fn copy(&self) -> SegmentLedger {
        SegmentLedger {
            code: self.code.clone(),
            title: self.title.clone(),
            tags: self.tags.clone(),
            retries: self.retries,
            name: self.name.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment ledger in status draft.
pub fn new_segment_ledger(code: &str, title: &str) -> SegmentLedger {
    SegmentLedger {
        code: code.to_string(),
        title: title.to_string(),
        tags: Vec::new(),
        retries: 0,
        name: "".to_string(),
        summary: "".to_string(),
        status: SegmentLedgerStatus::Draft,
    }
}
