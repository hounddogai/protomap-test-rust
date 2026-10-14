use super::*;

use acme_kit::text;

/// Where a default overage review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultOverageReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DefaultOverageReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DefaultOverageReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A default overage review as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultOverageReview {
    pub code: String,
    pub title: String,
    pub category: String,
    pub key_id: String,
    pub window_days: i64,
    pub source: String,
    pub threshold: f64,
    pub visible: bool,
    pub limit: i64,
    pub owner_id: String,
    pub status: DefaultOverageReviewStatus,
}

impl DefaultOverageReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultOverageReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultOverageReviewStatus) -> bool {
        match self.status {
            DefaultOverageReviewStatus::Requested => {
                match next {
                    DefaultOverageReviewStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultOverageReviewStatus::Granted => {
                return next == DefaultOverageReviewStatus::Expired;
            }
            DefaultOverageReviewStatus::Revoked => {
                match next {
                    DefaultOverageReviewStatus::Expired => {
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

    pub fn move_to(&mut self, next: DefaultOverageReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default overage review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.window_days as f64 + (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultOverageReview {
        DefaultOverageReview {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            key_id: self.key_id.clone(),
            window_days: self.window_days,
            source: self.source.clone(),
            threshold: self.threshold,
            visible: self.visible,
            limit: self.limit,
            owner_id: self.owner_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a default overage review in status requested.
pub fn new_default_overage_review(code: &str, title: &str) -> DefaultOverageReview {
    DefaultOverageReview {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        key_id: "".to_string(),
        window_days: 0,
        source: "".to_string(),
        threshold: 0.0,
        visible: false,
        limit: 0,
        owner_id: "".to_string(),
        status: DefaultOverageReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_default_overage_review("code-856", "title-701");
        assert_eq!(item.status, DefaultOverageReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_default_overage_review("code-856", "title-701");
        if item.move_to(DefaultOverageReviewStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultOverageReviewStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_default_overage_review("code-856", "title-701");
        item.status = DefaultOverageReviewStatus::Expired;
        assert!(
            item.move_to(DefaultOverageReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
