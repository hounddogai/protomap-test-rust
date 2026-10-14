use super::*;

use acme_kit::text;

/// Where a query audit is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryAuditStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QueryAuditStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for QueryAuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query audit. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryAudit {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub quantity: i64,
    pub duration_minutes: i64,
    pub status: QueryAuditStatus,
}

impl QueryAudit {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryAuditStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryAuditStatus) -> bool {
        match self.status {
            QueryAuditStatus::Idle => {
                match next {
                    QueryAuditStatus::Ready | QueryAuditStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryAuditStatus::Warming => {
                return next == QueryAuditStatus::Ready || next == QueryAuditStatus::Stopped;
            }
            QueryAuditStatus::Ready => {
                return next == QueryAuditStatus::Draining || next == QueryAuditStatus::Stopped;
            }
            QueryAuditStatus::Draining => {
                return next == QueryAuditStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryAuditStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query audit {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.quantity as f64 + (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryAudit {
        QueryAudit {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            quantity: self.quantity,
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query audit in status idle.
pub fn new_query_audit(code: &str, title: &str) -> QueryAudit {
    QueryAudit {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        quantity: 0,
        duration_minutes: 0,
        status: QueryAuditStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_query_audit("code-583", "title-166");
        assert_eq!(item.status, QueryAuditStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_query_audit("code-583", "title-166");
        if item.move_to(QueryAuditStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryAuditStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_query_audit("code-583", "title-166");
        item.status = QueryAuditStatus::Stopped;
        assert!(item.move_to(QueryAuditStatus::Idle).is_err(), "expected move to fail");
    }
}
