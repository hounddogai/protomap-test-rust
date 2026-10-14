use super::*;

use acme_kit::text;

/// Where a batched health check budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedHealthCheckBudgetStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedHealthCheckBudgetStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedHealthCheckBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One batched health check budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedHealthCheckBudget {
    pub code: String,
    pub label: String,
    pub title: String,
    pub quantity: i64,
    pub ceiling: f64,
    pub unit: String,
    pub host: String,
    pub ttl_seconds: i64,
    pub segment: String,
    pub country: String,
    pub timezone: String,
    pub status: BatchedHealthCheckBudgetStatus,
}

impl BatchedHealthCheckBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedHealthCheckBudgetStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedHealthCheckBudgetStatus) -> bool {
        match self.status {
            BatchedHealthCheckBudgetStatus::Created => {
                return next == BatchedHealthCheckBudgetStatus::Published;
            }
            BatchedHealthCheckBudgetStatus::Validated => {
                match next {
                    BatchedHealthCheckBudgetStatus::Published => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedHealthCheckBudgetStatus::Published => {
                return next == BatchedHealthCheckBudgetStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedHealthCheckBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched health check budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedHealthCheckBudget {
        BatchedHealthCheckBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            ceiling: self.ceiling,
            unit: self.unit.clone(),
            host: self.host.clone(),
            ttl_seconds: self.ttl_seconds,
            segment: self.segment.clone(),
            country: self.country.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a batched health check budget in status created.
pub fn new_batched_health_check_budget(code: &str, label: &str) -> BatchedHealthCheckBudget {
    BatchedHealthCheckBudget {
        code: code.to_string(),
        label: label.to_string(),
        title: "".to_string(),
        quantity: 0,
        ceiling: 0.0,
        unit: "".to_string(),
        host: "".to_string(),
        ttl_seconds: 0,
        segment: "".to_string(),
        country: "".to_string(),
        timezone: "".to_string(),
        status: BatchedHealthCheckBudgetStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_health_check_budget("code-510", "label-784");
        assert_eq!(item.status, BatchedHealthCheckBudgetStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_batched_health_check_budget("code-510", "label-784");
        if item.move_to(BatchedHealthCheckBudgetStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedHealthCheckBudgetStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_health_check_budget("code-510", "label-784");
        item.status = BatchedHealthCheckBudgetStatus::Retired;
        assert!(
            item.move_to(BatchedHealthCheckBudgetStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
