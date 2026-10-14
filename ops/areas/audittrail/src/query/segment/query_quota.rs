use super::*;

use acme_kit::text;

/// Where a query quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryQuotaStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QueryQuotaStatus {
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

impl std::fmt::Display for QueryQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryQuota {
    pub code: String,
    pub label: String,
    pub name: String,
    pub expires_at: i64,
    pub weight: f64,
    pub status: QueryQuotaStatus,
}

impl QueryQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryQuotaStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryQuotaStatus) -> bool {
        match self.status {
            QueryQuotaStatus::Draft => {
                match next {
                    QueryQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryQuotaStatus::Pending => {
                return next == QueryQuotaStatus::Suspended;
            }
            QueryQuotaStatus::Active => {
                match next {
                    QueryQuotaStatus::Suspended | QueryQuotaStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryQuotaStatus::Suspended => {
                match next {
                    QueryQuotaStatus::Closed => {
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

    pub fn move_to(&mut self, next: QueryQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query quota {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.expires_at as f64 + self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryQuota {
        QueryQuota {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            expires_at: self.expires_at,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a query quota in status draft.
pub fn new_query_quota(code: &str, label: &str) -> QueryQuota {
    QueryQuota {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        expires_at: 0,
        weight: 0.0,
        status: QueryQuotaStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_query_quota("code-162", "label-845");
        assert_eq!(item.status, QueryQuotaStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_query_quota("code-162", "label-845");
        if item.move_to(QueryQuotaStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryQuotaStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_query_quota("code-162", "label-845");
        item.status = QueryQuotaStatus::Closed;
        assert!(item.move_to(QueryQuotaStatus::Draft).is_err(), "expected move to fail");
    }
}
