use super::*;

use acme_kit::text;

/// Where a query estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryEstimateStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl QueryEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for QueryEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryEstimate {
    pub code: String,
    pub title: String,
    pub sequence: i64,
    pub floor_value: f64,
    pub minimum: f64,
    pub cipher: String,
    pub starts_at: i64,
    pub label: String,
    pub duration_minutes: i64,
    pub window_days: i64,
    pub capacity: i64,
    pub status: QueryEstimateStatus,
}

impl QueryEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryEstimateStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryEstimateStatus) -> bool {
        match self.status {
            QueryEstimateStatus::Requested => {
                return next == QueryEstimateStatus::Expired;
            }
            QueryEstimateStatus::Granted => {
                return next == QueryEstimateStatus::Revoked;
            }
            QueryEstimateStatus::Revoked => {
                return next == QueryEstimateStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: QueryEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "query estimate", self.code)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.sequence as f64 + self.floor_value) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryEstimate {
        QueryEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            sequence: self.sequence,
            floor_value: self.floor_value,
            minimum: self.minimum,
            cipher: self.cipher.clone(),
            starts_at: self.starts_at,
            label: self.label.clone(),
            duration_minutes: self.duration_minutes,
            window_days: self.window_days,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a query estimate in status requested.
pub fn new_query_estimate(code: &str, title: &str) -> QueryEstimate {
    QueryEstimate {
        code: code.to_string(),
        title: title.to_string(),
        sequence: 0,
        floor_value: 0.0,
        minimum: 0.0,
        cipher: "".to_string(),
        starts_at: 0,
        label: "".to_string(),
        duration_minutes: 0,
        window_days: 0,
        capacity: 0,
        status: QueryEstimateStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_query_estimate("code-745", "title-139");
        assert_eq!(item.status, QueryEstimateStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_expired() {
        let mut item = new_query_estimate("code-745", "title-139");
        if item.move_to(QueryEstimateStatus::Expired).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryEstimateStatus::Expired, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_query_estimate("code-745", "title-139");
        item.status = QueryEstimateStatus::Expired;
        assert!(
            item.move_to(QueryEstimateStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
