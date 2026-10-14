use super::*;

use acme_kit::text;

/// Where a segment digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentDigestStatus {
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

impl std::fmt::Display for SegmentDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a segment digest and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentDigest {
    pub code: String,
    pub title: String,
    pub batch_size: i64,
    pub principal: String,
    pub scope: String,
    pub priority: i64,
    pub timezone: String,
    pub burst: i64,
    pub key_id: String,
    pub status: SegmentDigestStatus,
}

impl SegmentDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentDigestStatus) -> bool {
        match self.status {
            SegmentDigestStatus::Open => {
                return next == SegmentDigestStatus::Reopened || next == SegmentDigestStatus::Archived;
            }
            SegmentDigestStatus::InReview => {
                return next == SegmentDigestStatus::Archived;
            }
            SegmentDigestStatus::Resolved => {
                match next {
                    SegmentDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentDigestStatus::Reopened => {
                return next == SegmentDigestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "segment digest", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.batch_size as f64) * (self.burst as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentDigest {
        SegmentDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            batch_size: self.batch_size,
            principal: self.principal.clone(),
            scope: self.scope.clone(),
            priority: self.priority,
            timezone: self.timezone.clone(),
            burst: self.burst,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a segment digest in status open.
pub fn new_segment_digest(code: &str, title: &str) -> SegmentDigest {
    SegmentDigest {
        code: code.to_string(),
        title: title.to_string(),
        batch_size: 0,
        principal: "".to_string(),
        scope: "".to_string(),
        priority: 0,
        timezone: "".to_string(),
        burst: 0,
        key_id: "".to_string(),
        status: SegmentDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_segment_digest("code-396", "title-259");
        assert_eq!(item.status, SegmentDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_segment_digest("code-396", "title-259");
        if item.move_to(SegmentDigestStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentDigestStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_segment_digest("code-396", "title-259");
        item.status = SegmentDigestStatus::Archived;
        assert!(
            item.move_to(SegmentDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
