use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a quote review is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuoteReviewStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl QuoteReviewStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for QuoteReviewStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a quote review and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct QuoteReview {
    pub code: String,
    pub name: String,
    pub source: String,
    pub scope: String,
    pub attributes: BTreeMap<String, String>,
    pub expires_at: i64,
    pub pinned: bool,
    pub status: QuoteReviewStatus,
}

impl QuoteReview {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuoteReviewStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuoteReviewStatus) -> bool {
        match self.status {
            QuoteReviewStatus::Requested => {
                match next {
                    QuoteReviewStatus::Granted | QuoteReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteReviewStatus::Granted => {
                match next {
                    QuoteReviewStatus::Revoked | QuoteReviewStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuoteReviewStatus::Revoked => {
                return next == QuoteReviewStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuoteReviewStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "quote review {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "quote review", self.code)
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

    pub fn measured_value(&self) -> f64 {
        self.expires_at as f64
    }

    pub fn copy(&self) -> QuoteReview {
        QuoteReview {
            code: self.code.clone(),
            name: self.name.clone(),
            source: self.source.clone(),
            scope: self.scope.clone(),
            attributes: self.attributes.clone(),
            expires_at: self.expires_at,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a quote review in status requested.
pub fn new_quote_review(code: &str, name: &str) -> QuoteReview {
    QuoteReview {
        code: code.to_string(),
        name: name.to_string(),
        source: "".to_string(),
        scope: "".to_string(),
        attributes: BTreeMap::new(),
        expires_at: 0,
        pinned: false,
        status: QuoteReviewStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_quote_review("code-988", "name-140");
        assert_eq!(item.status, QuoteReviewStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_quote_review("code-988", "name-140");
        if item.move_to(QuoteReviewStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuoteReviewStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_quote_review("code-988", "name-140");
        item.status = QuoteReviewStatus::Expired;
        assert!(
            item.move_to(QuoteReviewStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
