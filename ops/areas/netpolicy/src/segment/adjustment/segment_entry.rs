use super::*;

use acme_kit::text;

/// Where a segment entry is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentEntryStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl SegmentEntryStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for SegmentEntryStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One segment entry, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentEntry {
    pub code: String,
    pub label: String,
    pub name: String,
    pub ceiling: f64,
    pub ttl_seconds: i64,
    pub approved: bool,
    pub segment: String,
    pub host: String,
    pub ratio: f64,
    pub status: SegmentEntryStatus,
}

impl SegmentEntry {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentEntryStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentEntryStatus) -> bool {
        match self.status {
            SegmentEntryStatus::Requested => {
                match next {
                    SegmentEntryStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentEntryStatus::Granted => {
                match next {
                    SegmentEntryStatus::Revoked | SegmentEntryStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentEntryStatus::Revoked => {
                return next == SegmentEntryStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentEntryStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment entry {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ratio + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentEntry {
        SegmentEntry {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            ttl_seconds: self.ttl_seconds,
            approved: self.approved,
            segment: self.segment.clone(),
            host: self.host.clone(),
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a segment entry in status requested.
pub fn new_segment_entry(code: &str, label: &str) -> SegmentEntry {
    SegmentEntry {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        ceiling: 0.0,
        ttl_seconds: 0,
        approved: false,
        segment: "".to_string(),
        host: "".to_string(),
        ratio: 0.0,
        status: SegmentEntryStatus::Requested,
    }
}
