use super::*;

use acme_kit::text;

/// Where a query snapshot is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QuerySnapshotStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl QuerySnapshotStatus {
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

impl std::fmt::Display for QuerySnapshotStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A query snapshot as the query team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct QuerySnapshot {
    pub code: String,
    pub name: String,
    pub quota_units: i64,
    pub burst: i64,
    pub tenant_id: String,
    pub status: QuerySnapshotStatus,
}

impl QuerySnapshot {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QuerySnapshotStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QuerySnapshotStatus) -> bool {
        match self.status {
            QuerySnapshotStatus::Idle => {
                match next {
                    QuerySnapshotStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QuerySnapshotStatus::Warming => {
                return next == QuerySnapshotStatus::Draining || next == QuerySnapshotStatus::Stopped;
            }
            QuerySnapshotStatus::Ready => {
                return next == QuerySnapshotStatus::Draining || next == QuerySnapshotStatus::Stopped;
            }
            QuerySnapshotStatus::Draining => {
                return next == QuerySnapshotStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QuerySnapshotStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query snapshot {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.burst as f64 + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QuerySnapshot {
        QuerySnapshot {
            code: self.code.clone(),
            name: self.name.clone(),
            quota_units: self.quota_units,
            burst: self.burst,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a query snapshot in status idle.
pub fn new_query_snapshot(code: &str, name: &str) -> QuerySnapshot {
    QuerySnapshot {
        code: code.to_string(),
        name: name.to_string(),
        quota_units: 0,
        burst: 0,
        tenant_id: "".to_string(),
        status: QuerySnapshotStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_query_snapshot("code-208", "name-127");
        assert_eq!(item.status, QuerySnapshotStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_query_snapshot("code-208", "name-127");
        if item.move_to(QuerySnapshotStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QuerySnapshotStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_query_snapshot("code-208", "name-127");
        item.status = QuerySnapshotStatus::Stopped;
        assert!(
            item.move_to(QuerySnapshotStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
