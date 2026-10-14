use super::*;

use acme_kit::text;

/// Where a legacy zone review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyZoneReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl LegacyZoneReviewStatus {
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

impl std::fmt::Display for LegacyZoneReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One legacy zone review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyZoneReview {
    pub code: String,
    pub title: String,
    pub archived: bool,
    pub quota_units: i64,
    pub key_id: String,
    pub port: i64,
    pub status: LegacyZoneReviewStatus,
}

impl LegacyZoneReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyZoneReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyZoneReviewStatus) -> bool {
        match self.status {
            LegacyZoneReviewStatus::Open => {
                return next == LegacyZoneReviewStatus::Archived;
            }
            LegacyZoneReviewStatus::InReview => {
                match next {
                    LegacyZoneReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyZoneReviewStatus::Resolved => {
                match next {
                    LegacyZoneReviewStatus::Reopened | LegacyZoneReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyZoneReviewStatus::Reopened => {
                return next == LegacyZoneReviewStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LegacyZoneReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy zone review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.quota_units as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyZoneReview {
        LegacyZoneReview {
            code: self.code.clone(),
            title: self.title.clone(),
            archived: self.archived,
            quota_units: self.quota_units,
            key_id: self.key_id.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a legacy zone review in status open.
pub fn new_legacy_zone_review(code: &str, title: &str) -> LegacyZoneReview {
    LegacyZoneReview {
        code: code.to_string(),
        title: title.to_string(),
        archived: false,
        quota_units: 0,
        key_id: "".to_string(),
        port: 0,
        status: LegacyZoneReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_legacy_zone_review("code-517", "title-259");
        assert_eq!(item.status, LegacyZoneReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_archived() {
        let mut item = new_legacy_zone_review("code-517", "title-259");
        if item.move_to(LegacyZoneReviewStatus::Archived).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyZoneReviewStatus::Archived, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_legacy_zone_review("code-517", "title-259");
        item.status = LegacyZoneReviewStatus::Archived;
        assert!(
            item.move_to(LegacyZoneReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
