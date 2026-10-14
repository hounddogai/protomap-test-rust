use super::*;

use acme_kit::text;

/// Where a query tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryTierStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QueryTierStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QueryTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query tier. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryTier {
    pub code: String,
    pub title: String,
    pub ratio: f64,
    pub discount: f64,
    pub cipher: String,
    pub cost: f64,
    pub active: bool,
    pub ttl_seconds: i64,
    pub status: QueryTierStatus,
}

impl QueryTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryTierStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryTierStatus) -> bool {
        match self.status {
            QueryTierStatus::Proposed => {
                match next {
                    QueryTierStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryTierStatus::Approved => {
                return next == QueryTierStatus::Withdrawn;
            }
            QueryTierStatus::Rejected => {
                return next == QueryTierStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query tier {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.discount - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryTier {
        QueryTier {
            code: self.code.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            discount: self.discount,
            cipher: self.cipher.clone(),
            cost: self.cost,
            active: self.active,
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query tier in status proposed.
pub fn new_query_tier(code: &str, title: &str) -> QueryTier {
    QueryTier {
        code: code.to_string(),
        title: title.to_string(),
        ratio: 0.0,
        discount: 0.0,
        cipher: "".to_string(),
        cost: 0.0,
        active: false,
        ttl_seconds: 0,
        status: QueryTierStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_query_tier("code-292", "title-472");
        assert_eq!(item.status, QueryTierStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_query_tier("code-292", "title-472");
        if item.move_to(QueryTierStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryTierStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_query_tier("code-292", "title-472");
        item.status = QueryTierStatus::Withdrawn;
        assert!(
            item.move_to(QueryTierStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
