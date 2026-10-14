use super::*;

use acme_kit::text;

/// Where a segment notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentNoticeStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl SegmentNoticeStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for SegmentNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment notice as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentNotice {
    pub code: String,
    pub name: String,
    pub host: String,
    pub fingerprint: String,
    pub sequence: i64,
    pub attempts: i64,
    pub minimum: f64,
    pub status: SegmentNoticeStatus,
}

impl SegmentNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentNoticeStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentNoticeStatus) -> bool {
        match self.status {
            SegmentNoticeStatus::Proposed => {
                match next {
                    SegmentNoticeStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentNoticeStatus::Approved => {
                return next == SegmentNoticeStatus::Rejected;
            }
            SegmentNoticeStatus::Rejected => {
                return next == SegmentNoticeStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "segment notice", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.minimum - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentNotice {
        SegmentNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            fingerprint: self.fingerprint.clone(),
            sequence: self.sequence,
            attempts: self.attempts,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a segment notice in status proposed.
pub fn new_segment_notice(code: &str, name: &str) -> SegmentNotice {
    SegmentNotice {
        code: code.to_string(),
        name: name.to_string(),
        host: "".to_string(),
        fingerprint: "".to_string(),
        sequence: 0,
        attempts: 0,
        minimum: 0.0,
        status: SegmentNoticeStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_segment_notice("code-1", "name-960");
        assert_eq!(item.status, SegmentNoticeStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_segment_notice("code-1", "name-960");
        if item.move_to(SegmentNoticeStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentNoticeStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_segment_notice("code-1", "name-960");
        item.status = SegmentNoticeStatus::Withdrawn;
        assert!(
            item.move_to(SegmentNoticeStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
