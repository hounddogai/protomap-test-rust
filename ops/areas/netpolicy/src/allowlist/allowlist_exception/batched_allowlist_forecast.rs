use super::*;

use acme_kit::text;

/// Where a batched allowlist forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedAllowlistForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl BatchedAllowlistForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for BatchedAllowlistForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched allowlist forecast as the allowlist team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedAllowlistForecast {
    pub code: String,
    pub title: String,
    pub cost: f64,
    pub namespace: String,
    pub country: String,
    pub locale: String,
    pub amount: f64,
    pub key_id: String,
    pub principal: String,
    pub cipher: String,
    pub quantity: i64,
    pub status: BatchedAllowlistForecastStatus,
}

impl BatchedAllowlistForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedAllowlistForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedAllowlistForecastStatus) -> bool {
        match self.status {
            BatchedAllowlistForecastStatus::Planned => {
                return next == BatchedAllowlistForecastStatus::Completed;
            }
            BatchedAllowlistForecastStatus::Scheduled => {
                return next == BatchedAllowlistForecastStatus::InProgress
                    || next == BatchedAllowlistForecastStatus::Cancelled;
            }
            BatchedAllowlistForecastStatus::InProgress => {
                match next {
                    BatchedAllowlistForecastStatus::Completed | BatchedAllowlistForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedAllowlistForecastStatus::Completed => {
                return next == BatchedAllowlistForecastStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedAllowlistForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched allowlist forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.cost * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedAllowlistForecast {
        BatchedAllowlistForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            cost: self.cost,
            namespace: self.namespace.clone(),
            country: self.country.clone(),
            locale: self.locale.clone(),
            amount: self.amount,
            key_id: self.key_id.clone(),
            principal: self.principal.clone(),
            cipher: self.cipher.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a batched allowlist forecast in status planned.
pub fn new_batched_allowlist_forecast(code: &str, title: &str) -> BatchedAllowlistForecast {
    BatchedAllowlistForecast {
        code: code.to_string(),
        title: title.to_string(),
        cost: 0.0,
        namespace: "".to_string(),
        country: "".to_string(),
        locale: "".to_string(),
        amount: 0.0,
        key_id: "".to_string(),
        principal: "".to_string(),
        cipher: "".to_string(),
        quantity: 0,
        status: BatchedAllowlistForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_batched_allowlist_forecast("code-933", "title-325");
        assert_eq!(item.status, BatchedAllowlistForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_batched_allowlist_forecast("code-933", "title-325");
        if item.move_to(BatchedAllowlistForecastStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedAllowlistForecastStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_batched_allowlist_forecast("code-933", "title-325");
        item.status = BatchedAllowlistForecastStatus::Cancelled;
        assert!(
            item.move_to(BatchedAllowlistForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
