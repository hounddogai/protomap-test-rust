use super::*;

use acme_kit::text;

/// Where a default release review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultReleaseReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl DefaultReleaseReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for DefaultReleaseReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One default release review, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultReleaseReview {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub country: String,
    pub port: i64,
    pub status: DefaultReleaseReviewStatus,
}

impl DefaultReleaseReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultReleaseReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultReleaseReviewStatus) -> bool {
        match self.status {
            DefaultReleaseReviewStatus::Draft => {
                return next == DefaultReleaseReviewStatus::Closed;
            }
            DefaultReleaseReviewStatus::Pending => {
                match next {
                    DefaultReleaseReviewStatus::Active | DefaultReleaseReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseReviewStatus::Active => {
                match next {
                    DefaultReleaseReviewStatus::Suspended | DefaultReleaseReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultReleaseReviewStatus::Suspended => {
                return next == DefaultReleaseReviewStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultReleaseReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default release review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "default release review", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.cost + (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultReleaseReview {
        DefaultReleaseReview {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            country: self.country.clone(),
            port: self.port,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a default release review in status draft.
pub fn new_default_release_review(code: &str, title: &str) -> DefaultReleaseReview {
    DefaultReleaseReview {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        country: "".to_string(),
        port: 0,
        status: DefaultReleaseReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_default_release_review("code-789", "title-881");
        assert_eq!(item.status, DefaultReleaseReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_default_release_review("code-789", "title-881");
        if item.move_to(DefaultReleaseReviewStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultReleaseReviewStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_default_release_review("code-789", "title-881");
        item.status = DefaultReleaseReviewStatus::Closed;
        assert!(
            item.move_to(DefaultReleaseReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
