use super::*;

use acme_kit::text;

/// Where a query claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryClaimStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QueryClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QueryClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryClaim {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub currency: String,
    pub updated_at: i64,
    pub reviewer: String,
    pub checksum: String,
    pub scope: String,
    pub status: QueryClaimStatus,
}

impl QueryClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryClaimStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryClaimStatus) -> bool {
        match self.status {
            QueryClaimStatus::Proposed => {
                match next {
                    QueryClaimStatus::Rejected | QueryClaimStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryClaimStatus::Approved => {
                match next {
                    QueryClaimStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryClaimStatus::Rejected => {
                return next == QueryClaimStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query claim", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> QueryClaim {
        QueryClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            currency: self.currency.clone(),
            updated_at: self.updated_at,
            reviewer: self.reviewer.clone(),
            checksum: self.checksum.clone(),
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a query claim in status proposed.
pub fn new_query_claim(code: &str, title: &str) -> QueryClaim {
    QueryClaim {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        currency: "".to_string(),
        updated_at: 0,
        reviewer: "".to_string(),
        checksum: "".to_string(),
        scope: "".to_string(),
        status: QueryClaimStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_query_claim("code-62", "title-754");
        assert_eq!(item.status, QueryClaimStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_query_claim("code-62", "title-754");
        if item.move_to(QueryClaimStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryClaimStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_query_claim("code-62", "title-754");
        item.status = QueryClaimStatus::Withdrawn;
        assert!(
            item.move_to(QueryClaimStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
