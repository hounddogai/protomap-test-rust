use super::*;

use acme_kit::text;

/// Where a segment exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SegmentExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SegmentExceptionStatus {
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

impl std::fmt::Display for SegmentExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a segment exception and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct SegmentException {
    pub code: String,
    pub title: String,
    pub discount: f64,
    pub expires_at: i64,
    pub owner_id: String,
    pub pinned: bool,
    pub status: SegmentExceptionStatus,
}

impl SegmentException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SegmentExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SegmentExceptionStatus) -> bool {
        match self.status {
            SegmentExceptionStatus::Open => {
                return next == SegmentExceptionStatus::Resolved;
            }
            SegmentExceptionStatus::InReview => {
                match next {
                    SegmentExceptionStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SegmentExceptionStatus::Resolved => {
                return next == SegmentExceptionStatus::Archived;
            }
            SegmentExceptionStatus::Reopened => {
                return next == SegmentExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SegmentExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "segment exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "segment exception", self.code)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.discount - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SegmentException {
        SegmentException {
            code: self.code.clone(),
            title: self.title.clone(),
            discount: self.discount,
            expires_at: self.expires_at,
            owner_id: self.owner_id.clone(),
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a segment exception in status open.
pub fn new_segment_exception(code: &str, title: &str) -> SegmentException {
    SegmentException {
        code: code.to_string(),
        title: title.to_string(),
        discount: 0.0,
        expires_at: 0,
        owner_id: "".to_string(),
        pinned: false,
        status: SegmentExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_segment_exception("code-690", "title-909");
        assert_eq!(item.status, SegmentExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_segment_exception("code-690", "title-909");
        if item.move_to(SegmentExceptionStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SegmentExceptionStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_segment_exception("code-690", "title-909");
        item.status = SegmentExceptionStatus::Archived;
        assert!(
            item.move_to(SegmentExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
