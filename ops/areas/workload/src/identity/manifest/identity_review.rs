use super::*;

use acme_kit::text;

/// Where a identity review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum IdentityReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl IdentityReviewStatus {
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

impl std::fmt::Display for IdentityReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A identity review as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct IdentityReview {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub floor_value: f64,
    pub title: String,
    pub batch_size: i64,
    pub active: bool,
    pub status: IdentityReviewStatus,
}

impl IdentityReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            IdentityReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: IdentityReviewStatus) -> bool {
        match self.status {
            IdentityReviewStatus::Draft => {
                match next {
                    IdentityReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityReviewStatus::Pending => {
                match next {
                    IdentityReviewStatus::Active | IdentityReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityReviewStatus::Active => {
                match next {
                    IdentityReviewStatus::Suspended | IdentityReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            IdentityReviewStatus::Suspended => {
                return next == IdentityReviewStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: IdentityReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "identity review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.batch_size as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> IdentityReview {
        IdentityReview {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            floor_value: self.floor_value,
            title: self.title.clone(),
            batch_size: self.batch_size,
            active: self.active,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a identity review in status draft.
pub fn new_identity_review(code: &str, name: &str) -> IdentityReview {
    IdentityReview {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        floor_value: 0.0,
        title: "".to_string(),
        batch_size: 0,
        active: false,
        status: IdentityReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_identity_review("code-621", "name-559");
        assert_eq!(item.status, IdentityReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_identity_review("code-621", "name-559");
        if item.move_to(IdentityReviewStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, IdentityReviewStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_identity_review("code-621", "name-559");
        item.status = IdentityReviewStatus::Closed;
        assert!(
            item.move_to(IdentityReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
