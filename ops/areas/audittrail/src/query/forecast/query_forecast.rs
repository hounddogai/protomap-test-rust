use super::*;

use acme_kit::text;

/// Where a query forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum QueryForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl QueryForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for QueryForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One query forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct QueryForecast {
    pub code: String,
    pub title: String,
    pub cipher: String,
    pub revision: i64,
    pub retries: i64,
    pub capacity: i64,
    pub country: String,
    pub created_at: i64,
    pub audience: String,
    pub ceiling: f64,
    pub status: QueryForecastStatus,
}

impl QueryForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            QueryForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: QueryForecastStatus) -> bool {
        match self.status {
            QueryForecastStatus::New => {
                return next == QueryForecastStatus::Queued || next == QueryForecastStatus::Done;
            }
            QueryForecastStatus::Queued => {
                match next {
                    QueryForecastStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            QueryForecastStatus::Running => {
                return next == QueryForecastStatus::Failed;
            }
            QueryForecastStatus::Done => {
                match next {
                    QueryForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: QueryForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "query forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.capacity as f64 - self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> QueryForecast {
        QueryForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            cipher: self.cipher.clone(),
            revision: self.revision,
            retries: self.retries,
            capacity: self.capacity,
            country: self.country.clone(),
            created_at: self.created_at,
            audience: self.audience.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a query forecast in status new.
pub fn new_query_forecast(code: &str, title: &str) -> QueryForecast {
    QueryForecast {
        code: code.to_string(),
        title: title.to_string(),
        cipher: "".to_string(),
        revision: 0,
        retries: 0,
        capacity: 0,
        country: "".to_string(),
        created_at: 0,
        audience: "".to_string(),
        ceiling: 0.0,
        status: QueryForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_query_forecast("code-97", "title-408");
        assert_eq!(item.status, QueryForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_query_forecast("code-97", "title-408");
        if item.move_to(QueryForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, QueryForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_query_forecast("code-97", "title-408");
        item.status = QueryForecastStatus::Failed;
        assert!(item.move_to(QueryForecastStatus::New).is_err(), "expected move to fail");
    }
}
