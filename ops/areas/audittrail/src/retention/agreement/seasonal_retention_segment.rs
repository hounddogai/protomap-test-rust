use super::*;

use acme_kit::text;

/// Where a seasonal retention segment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalRetentionSegmentStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl SeasonalRetentionSegmentStatus {
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

impl std::fmt::Display for SeasonalRetentionSegmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal retention segment, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalRetentionSegment {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub ttl_seconds: i64,
    pub tenant_id: String,
    pub cipher: String,
    pub source: String,
    pub external_ref: String,
    pub status: SeasonalRetentionSegmentStatus,
}

impl SeasonalRetentionSegment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalRetentionSegmentStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalRetentionSegmentStatus) -> bool {
        match self.status {
            SeasonalRetentionSegmentStatus::Open => {
                match next {
                    SeasonalRetentionSegmentStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRetentionSegmentStatus::InReview => {
                match next {
                    SeasonalRetentionSegmentStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalRetentionSegmentStatus::Resolved => {
                return next == SeasonalRetentionSegmentStatus::Archived;
            }
            SeasonalRetentionSegmentStatus::Reopened => {
                return next == SeasonalRetentionSegmentStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: SeasonalRetentionSegmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal retention segment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalRetentionSegment {
        SeasonalRetentionSegment {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            ttl_seconds: self.ttl_seconds,
            tenant_id: self.tenant_id.clone(),
            cipher: self.cipher.clone(),
            source: self.source.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a seasonal retention segment in status open.
pub fn new_seasonal_retention_segment(code: &str, title: &str) -> SeasonalRetentionSegment {
    SeasonalRetentionSegment {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        ttl_seconds: 0,
        tenant_id: "".to_string(),
        cipher: "".to_string(),
        source: "".to_string(),
        external_ref: "".to_string(),
        status: SeasonalRetentionSegmentStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_seasonal_retention_segment("code-884", "title-290");
        assert_eq!(item.status, SeasonalRetentionSegmentStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_seasonal_retention_segment("code-884", "title-290");
        if item.move_to(SeasonalRetentionSegmentStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalRetentionSegmentStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_seasonal_retention_segment("code-884", "title-290");
        item.status = SeasonalRetentionSegmentStatus::Archived;
        assert!(
            item.move_to(SeasonalRetentionSegmentStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
