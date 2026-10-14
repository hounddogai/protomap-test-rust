use super::*;

use acme_kit::text;

/// Where a postmortem digest is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemDigestStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PostmortemDigestStatus {
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

impl std::fmt::Display for PostmortemDigestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem digest as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemDigest {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub created_at: i64,
    pub starts_at: i64,
    pub locale: String,
    pub retries: i64,
    pub pinned: bool,
    pub status: PostmortemDigestStatus,
}

impl PostmortemDigest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemDigestStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemDigestStatus) -> bool {
        match self.status {
            PostmortemDigestStatus::Open => {
                return next == PostmortemDigestStatus::Resolved || next == PostmortemDigestStatus::Reopened;
            }
            PostmortemDigestStatus::InReview => {
                match next {
                    PostmortemDigestStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemDigestStatus::Resolved => {
                match next {
                    PostmortemDigestStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemDigestStatus::Reopened => {
                return next == PostmortemDigestStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemDigestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem digest {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.created_at as f64) * (self.starts_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemDigest {
        PostmortemDigest {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            created_at: self.created_at,
            starts_at: self.starts_at,
            locale: self.locale.clone(),
            retries: self.retries,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a postmortem digest in status open.
pub fn new_postmortem_digest(code: &str, title: &str) -> PostmortemDigest {
    PostmortemDigest {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        created_at: 0,
        starts_at: 0,
        locale: "".to_string(),
        retries: 0,
        pinned: false,
        status: PostmortemDigestStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_postmortem_digest("code-632", "title-145");
        assert_eq!(item.status, PostmortemDigestStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_postmortem_digest("code-632", "title-145");
        if item.move_to(PostmortemDigestStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemDigestStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_postmortem_digest("code-632", "title-145");
        item.status = PostmortemDigestStatus::Archived;
        assert!(
            item.move_to(PostmortemDigestStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
