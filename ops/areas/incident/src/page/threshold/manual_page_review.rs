use super::*;

use acme_kit::text;

/// Where a manual page review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ManualPageReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ManualPageReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ManualPageReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One manual page review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ManualPageReview {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub approved: bool,
    pub tags: Vec<String>,
    pub description: String,
    pub version: i64,
    pub created_at: i64,
    pub archived: bool,
    pub retries: i64,
    pub rotation_days: i64,
    pub status: ManualPageReviewStatus,
}

impl ManualPageReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ManualPageReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ManualPageReviewStatus) -> bool {
        match self.status {
            ManualPageReviewStatus::Requested => {
                return next == ManualPageReviewStatus::Granted || next == ManualPageReviewStatus::Revoked;
            }
            ManualPageReviewStatus::Granted => {
                return next == ManualPageReviewStatus::Revoked || next == ManualPageReviewStatus::Expired;
            }
            ManualPageReviewStatus::Revoked => {
                match next {
                    ManualPageReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: ManualPageReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "manual page review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.version as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ManualPageReview {
        ManualPageReview {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            approved: self.approved,
            tags: self.tags.clone(),
            description: self.description.clone(),
            version: self.version,
            created_at: self.created_at,
            archived: self.archived,
            retries: self.retries,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a manual page review in status requested.
pub fn new_manual_page_review(code: &str, title: &str) -> ManualPageReview {
    ManualPageReview {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        approved: false,
        tags: Vec::new(),
        description: "".to_string(),
        version: 0,
        created_at: 0,
        archived: false,
        retries: 0,
        rotation_days: 0,
        status: ManualPageReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_manual_page_review("code-365", "title-74");
        assert_eq!(item.status, ManualPageReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_manual_page_review("code-365", "title-74");
        if item.move_to(ManualPageReviewStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ManualPageReviewStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_manual_page_review("code-365", "title-74");
        item.status = ManualPageReviewStatus::Expired;
        assert!(
            item.move_to(ManualPageReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
