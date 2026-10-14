use super::*;

use acme_kit::text;

/// Where a postmortem exception is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PostmortemExceptionStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PostmortemExceptionStatus {
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

impl std::fmt::Display for PostmortemExceptionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A postmortem exception. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PostmortemException {
    pub code: String,
    pub name: String,
    pub ratio: f64,
    pub cost: f64,
    pub margin: f64,
    pub status: PostmortemExceptionStatus,
}

impl PostmortemException {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PostmortemExceptionStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PostmortemExceptionStatus) -> bool {
        match self.status {
            PostmortemExceptionStatus::Open => {
                return next == PostmortemExceptionStatus::InReview || next == PostmortemExceptionStatus::Reopened;
            }
            PostmortemExceptionStatus::InReview => {
                match next {
                    PostmortemExceptionStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PostmortemExceptionStatus::Resolved => {
                return next == PostmortemExceptionStatus::Reopened || next == PostmortemExceptionStatus::Archived;
            }
            PostmortemExceptionStatus::Reopened => {
                return next == PostmortemExceptionStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PostmortemExceptionStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "postmortem exception {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.margin - self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PostmortemException {
        PostmortemException {
            code: self.code.clone(),
            name: self.name.clone(),
            ratio: self.ratio,
            cost: self.cost,
            margin: self.margin,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a postmortem exception in status open.
pub fn new_postmortem_exception(code: &str, name: &str) -> PostmortemException {
    PostmortemException {
        code: code.to_string(),
        name: name.to_string(),
        ratio: 0.0,
        cost: 0.0,
        margin: 0.0,
        status: PostmortemExceptionStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_postmortem_exception("code-147", "name-44");
        assert_eq!(item.status, PostmortemExceptionStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_postmortem_exception("code-147", "name-44");
        if item.move_to(PostmortemExceptionStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PostmortemExceptionStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_postmortem_exception("code-147", "name-44");
        item.status = PostmortemExceptionStatus::Archived;
        assert!(
            item.move_to(PostmortemExceptionStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
