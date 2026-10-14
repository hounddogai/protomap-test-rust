use super::*;

use acme_kit::text;

/// Where a query notice is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryNoticeStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl QueryNoticeStatus {
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

impl std::fmt::Display for QueryNoticeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query notice. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryNotice {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub margin: f64,
    pub batch_size: i64,
    pub title: String,
    pub cost: f64,
    pub visible: bool,
    pub version: i64,
    pub status: QueryNoticeStatus,
}

impl QueryNotice {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryNoticeStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryNoticeStatus) -> bool {
        match self.status {
            QueryNoticeStatus::Draft => {
                match next {
                    QueryNoticeStatus::Suspended => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryNoticeStatus::Pending => {
                return next == QueryNoticeStatus::Suspended || next == QueryNoticeStatus::Closed;
            }
            QueryNoticeStatus::Active => {
                return next == QueryNoticeStatus::Suspended;
            }
            QueryNoticeStatus::Suspended => {
                match next {
                    QueryNoticeStatus::Closed => {
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

    pub fn move_to(&mut self, next: QueryNoticeStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query notice {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "query notice", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost + (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryNotice {
        QueryNotice {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            margin: self.margin,
            batch_size: self.batch_size,
            title: self.title.clone(),
            cost: self.cost,
            visible: self.visible,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a query notice in status draft.
pub fn new_query_notice(code: &str, name: &str) -> QueryNotice {
    QueryNotice {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        margin: 0.0,
        batch_size: 0,
        title: "".to_string(),
        cost: 0.0,
        visible: false,
        version: 0,
        status: QueryNoticeStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_query_notice("code-462", "name-515");
        assert_eq!(item.status, QueryNoticeStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_suspended() {
        let mut item = new_query_notice("code-462", "name-515");
        if item.move_to(QueryNoticeStatus::Suspended).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryNoticeStatus::Suspended, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_query_notice("code-462", "name-515");
        item.status = QueryNoticeStatus::Closed;
        assert!(item.move_to(QueryNoticeStatus::Draft).is_err(), "expected move to fail");
    }
}
