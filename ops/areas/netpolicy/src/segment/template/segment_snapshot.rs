use super::*;

use acme_kit::text;

/// Where a segment snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentSnapshotStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentSnapshotStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for SegmentSnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment snapshot as the segment team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentSnapshot {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub currency: String,
    pub limit: i64,
    pub quota_units: i64,
    pub status: SegmentSnapshotStatus,
}

impl SegmentSnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentSnapshotStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentSnapshotStatus) -> bool {
        match self.status {
            SegmentSnapshotStatus::Open => {
                match next {
                    SegmentSnapshotStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentSnapshotStatus::InReview => {
                match next {
                    SegmentSnapshotStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentSnapshotStatus::Resolved => {
                match next {
                    SegmentSnapshotStatus::Reopened | SegmentSnapshotStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentSnapshotStatus::Reopened => {
                return next == SegmentSnapshotStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentSnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn net_volume(&self) -> f64 {
        ((self.limit as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentSnapshot {
        SegmentSnapshot {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            currency: self.currency.clone(),
            limit: self.limit,
            quota_units: self.quota_units,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a segment snapshot in status open.
pub fn new_segment_snapshot(code: &str, label: &str) -> SegmentSnapshot {
    SegmentSnapshot {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        currency: "".to_string(),
        limit: 0,
        quota_units: 0,
        status: SegmentSnapshotStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_segment_snapshot("code-923", "label-21");
        assert_eq!(item.status, SegmentSnapshotStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_segment_snapshot("code-923", "label-21");
        if item.move_to(SegmentSnapshotStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentSnapshotStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_segment_snapshot("code-923", "label-21");
        item.status = SegmentSnapshotStatus::Archived;
        assert!(
            item.move_to(SegmentSnapshotStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
