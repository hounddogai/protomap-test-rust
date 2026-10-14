use super::*;

use acme_kit::text;

/// Where a release review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseReviewStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl ReleaseReviewStatus {
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

impl std::fmt::Display for ReleaseReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a release review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseReview {
    pub code: String,
    pub title: String,
    pub fingerprint: String,
    pub updated_at: i64,
    pub checksum: String,
    pub key_id: String,
    pub status: ReleaseReviewStatus,
}

impl ReleaseReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseReviewStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseReviewStatus) -> bool {
        match self.status {
            ReleaseReviewStatus::Open => {
                match next {
                    ReleaseReviewStatus::InReview | ReleaseReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseReviewStatus::InReview => {
                return next == ReleaseReviewStatus::Resolved || next == ReleaseReviewStatus::Reopened;
            }
            ReleaseReviewStatus::Resolved => {
                match next {
                    ReleaseReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseReviewStatus::Reopened => {
                match next {
                    ReleaseReviewStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReleaseReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> ReleaseReview {
        ReleaseReview {
            code: self.code.clone(),
            title: self.title.clone(),
            fingerprint: self.fingerprint.clone(),
            updated_at: self.updated_at,
            checksum: self.checksum.clone(),
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a release review in status open.
pub fn new_release_review(code: &str, title: &str) -> ReleaseReview {
    ReleaseReview {
        code: code.to_string(),
        title: title.to_string(),
        fingerprint: "".to_string(),
        updated_at: 0,
        checksum: "".to_string(),
        key_id: "".to_string(),
        status: ReleaseReviewStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_release_review("code-386", "title-811");
        assert_eq!(item.status, ReleaseReviewStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_release_review("code-386", "title-811");
        if item.move_to(ReleaseReviewStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ReleaseReviewStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_release_review("code-386", "title-811");
        item.status = ReleaseReviewStatus::Archived;
        assert!(
            item.move_to(ReleaseReviewStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
