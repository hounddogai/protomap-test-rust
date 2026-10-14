use super::*;

use acme_kit::text;

/// Where a segment baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentBaselineStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentBaselineStatus {
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

impl std::fmt::Display for SegmentBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A segment baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentBaseline {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub ratio: f64,
    pub archived: bool,
    pub duration_minutes: i64,
    pub status: SegmentBaselineStatus,
}

impl SegmentBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentBaselineStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentBaselineStatus) -> bool {
        match self.status {
            SegmentBaselineStatus::Open => {
                return next == SegmentBaselineStatus::Reopened;
            }
            SegmentBaselineStatus::InReview => {
                return next == SegmentBaselineStatus::Resolved || next == SegmentBaselineStatus::Reopened;
            }
            SegmentBaselineStatus::Resolved => {
                match next {
                    SegmentBaselineStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentBaselineStatus::Reopened => {
                return next == SegmentBaselineStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.duration_minutes as f64 - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentBaseline {
        SegmentBaseline {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            ratio: self.ratio,
            archived: self.archived,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment baseline in status open.
pub fn new_segment_baseline(code: &str, title: &str) -> SegmentBaseline {
    SegmentBaseline {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        ratio: 0.0,
        archived: false,
        duration_minutes: 0,
        status: SegmentBaselineStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_segment_baseline("code-871", "title-821");
        assert_eq!(item.status, SegmentBaselineStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_segment_baseline("code-871", "title-821");
        if item.move_to(SegmentBaselineStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentBaselineStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_segment_baseline("code-871", "title-821");
        item.status = SegmentBaselineStatus::Archived;
        assert!(
            item.move_to(SegmentBaselineStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
