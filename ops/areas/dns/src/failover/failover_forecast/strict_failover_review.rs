use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a strict failover review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum StrictFailoverReviewStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl StrictFailoverReviewStatus {
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

impl std::fmt::Display for StrictFailoverReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A strict failover review. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct StrictFailoverReview {
    pub code: String,
    pub title: String,
    pub key_id: String,
    pub namespace: String,
    pub priority: i64,
    pub attributes: BTreeMap<String, String>,
    pub retries: i64,
    pub ttl_seconds: i64,
    pub status: StrictFailoverReviewStatus,
}

impl StrictFailoverReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            StrictFailoverReviewStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: StrictFailoverReviewStatus) -> bool {
        match self.status {
            StrictFailoverReviewStatus::Draft => {
                return next == StrictFailoverReviewStatus::Active;
            }
            StrictFailoverReviewStatus::Pending => {
                return next == StrictFailoverReviewStatus::Suspended;
            }
            StrictFailoverReviewStatus::Active => {
                match next {
                    StrictFailoverReviewStatus::Suspended | StrictFailoverReviewStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            StrictFailoverReviewStatus::Suspended => {
                match next {
                    StrictFailoverReviewStatus::Closed => {
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

    pub fn move_to(&mut self, next: StrictFailoverReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "strict failover review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn effective_total(&self) -> f64 {
        ((self.priority as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> StrictFailoverReview {
        StrictFailoverReview {
            code: self.code.clone(),
            title: self.title.clone(),
            key_id: self.key_id.clone(),
            namespace: self.namespace.clone(),
            priority: self.priority,
            attributes: self.attributes.clone(),
            retries: self.retries,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a strict failover review in status draft.
pub fn new_strict_failover_review(code: &str, title: &str) -> StrictFailoverReview {
    StrictFailoverReview {
        code: code.to_string(),
        title: title.to_string(),
        key_id: "".to_string(),
        namespace: "".to_string(),
        priority: 0,
        attributes: BTreeMap::new(),
        retries: 0,
        ttl_seconds: 0,
        status: StrictFailoverReviewStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_strict_failover_review("code-357", "title-756");
        assert_eq!(item.status, StrictFailoverReviewStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_strict_failover_review("code-357", "title-756");
        if item.move_to(StrictFailoverReviewStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, StrictFailoverReviewStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_strict_failover_review("code-357", "title-756");
        item.status = StrictFailoverReviewStatus::Closed;
        assert!(
            item.move_to(StrictFailoverReviewStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
