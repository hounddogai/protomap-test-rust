use super::*;

use acme_kit::text;

/// Where a federation review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl FederationReviewStatus {
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

impl std::fmt::Display for FederationReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationReview {
    pub code: String,
    pub name: String,
    pub label: String,
    pub duration_minutes: i64,
    pub limit: i64,
    pub currency: String,
    pub contact_email: String,
    pub ttl_seconds: i64,
    pub threshold: f64,
    pub tags: Vec<String>,
    pub discount: f64,
    pub status: FederationReviewStatus,
}

impl FederationReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationReviewStatus) -> bool {
        match self.status {
            FederationReviewStatus::Draft => {
                match next {
                    FederationReviewStatus::Pending | FederationReviewStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationReviewStatus::Pending => {
                return next == FederationReviewStatus::Suspended;
            }
            FederationReviewStatus::Active => {
                return next == FederationReviewStatus::Suspended;
            }
            FederationReviewStatus::Suspended => {
                match next {
                    FederationReviewStatus::Closed => {
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

    pub fn move_to(&mut self, next: FederationReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "federation review", self.code)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_total(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationReview {
        FederationReview {
            code: self.code.clone(),
            name: self.name.clone(),
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            limit: self.limit,
            currency: self.currency.clone(),
            contact_email: self.contact_email.clone(),
            ttl_seconds: self.ttl_seconds,
            threshold: self.threshold,
            tags: self.tags.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a federation review in status draft.
pub fn new_federation_review(code: &str, name: &str) -> FederationReview {
    FederationReview {
        code: code.to_string(),
        name: name.to_string(),
        label: "".to_string(),
        duration_minutes: 0,
        limit: 0,
        currency: "".to_string(),
        contact_email: "".to_string(),
        ttl_seconds: 0,
        threshold: 0.0,
        tags: Vec::new(),
        discount: 0.0,
        status: FederationReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_federation_review("code-589", "name-818");
        assert_eq!(item.status, FederationReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_federation_review("code-589", "name-818");
        if item.move_to(FederationReviewStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationReviewStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_federation_review("code-589", "name-818");
        item.status = FederationReviewStatus::Closed;
        assert!(
            item.move_to(FederationReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
