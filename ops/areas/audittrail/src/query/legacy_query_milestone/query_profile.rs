use super::*;

use acme_kit::text;

/// Where a query profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryProfileStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl QueryProfileStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for QueryProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query profile. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryProfile {
    pub code: String,
    pub name: String,
    pub region: String,
    pub expires_at: i64,
    pub discount: f64,
    pub scope: String,
    pub quota_units: i64,
    pub title: String,
    pub status: QueryProfileStatus,
}

impl QueryProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryProfileStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryProfileStatus) -> bool {
        match self.status {
            QueryProfileStatus::Proposed => {
                match next {
                    QueryProfileStatus::Rejected | QueryProfileStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryProfileStatus::Approved => {
                return next == QueryProfileStatus::Rejected;
            }
            QueryProfileStatus::Rejected => {
                return next == QueryProfileStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query profile {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.expires_at as f64) * (self.quota_units as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryProfile {
        QueryProfile {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            expires_at: self.expires_at,
            discount: self.discount,
            scope: self.scope.clone(),
            quota_units: self.quota_units,
            title: self.title.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a query profile in status proposed.
pub fn new_query_profile(code: &str, name: &str) -> QueryProfile {
    QueryProfile {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        expires_at: 0,
        discount: 0.0,
        scope: "".to_string(),
        quota_units: 0,
        title: "".to_string(),
        status: QueryProfileStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_query_profile("code-945", "name-952");
        assert_eq!(item.status, QueryProfileStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_query_profile("code-945", "name-952");
        if item.move_to(QueryProfileStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryProfileStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_query_profile("code-945", "name-952");
        item.status = QueryProfileStatus::Withdrawn;
        assert!(
            item.move_to(QueryProfileStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
