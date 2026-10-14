use super::*;

use acme_kit::text;

/// Where a segment record is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentRecordStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl SegmentRecordStatus {
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

impl std::fmt::Display for SegmentRecordStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment record as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentRecord {
    pub code: String,
    pub label: String,
    pub duration_minutes: i64,
    pub retries: i64,
    pub limit: i64,
    pub burst: i64,
    pub weight: f64,
    pub phone: String,
    pub status: SegmentRecordStatus,
}

impl SegmentRecord {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentRecordStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentRecordStatus) -> bool {
        match self.status {
            SegmentRecordStatus::Draft => {
                return next == SegmentRecordStatus::Pending || next == SegmentRecordStatus::Closed;
            }
            SegmentRecordStatus::Pending => {
                match next {
                    SegmentRecordStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentRecordStatus::Active => {
                return next == SegmentRecordStatus::Suspended;
            }
            SegmentRecordStatus::Suspended => {
                return next == SegmentRecordStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentRecordStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment record {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "segment record", self.code)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.retries as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentRecord {
        SegmentRecord {
            code: self.code.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            retries: self.retries,
            limit: self.limit,
            burst: self.burst,
            weight: self.weight,
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a segment record in status draft.
pub fn new_segment_record(code: &str, label: &str) -> SegmentRecord {
    SegmentRecord {
        code: code.to_string(),
        label: label.to_string(),
        duration_minutes: 0,
        retries: 0,
        limit: 0,
        burst: 0,
        weight: 0.0,
        phone: "".to_string(),
        status: SegmentRecordStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_segment_record("code-72", "label-819");
        assert_eq!(item.status, SegmentRecordStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_segment_record("code-72", "label-819");
        if item.move_to(SegmentRecordStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentRecordStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_segment_record("code-72", "label-819");
        item.status = SegmentRecordStatus::Closed;
        assert!(
            item.move_to(SegmentRecordStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
