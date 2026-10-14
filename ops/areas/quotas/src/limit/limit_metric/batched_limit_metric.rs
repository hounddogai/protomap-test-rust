use super::*;

use acme_kit::text;

/// Where a batched limit metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedLimitMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl BatchedLimitMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for BatchedLimitMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched limit metric as the limit team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedLimitMetric {
    pub code: String,
    pub name: String,
    pub quantity: i64,
    pub amount: f64,
    pub principal: String,
    pub revision: i64,
    pub starts_at: i64,
    pub expires_at: i64,
    pub status: BatchedLimitMetricStatus,
}

impl BatchedLimitMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedLimitMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedLimitMetricStatus) -> bool {
        match self.status {
            BatchedLimitMetricStatus::Created => {
                match next {
                    BatchedLimitMetricStatus::Validated => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedLimitMetricStatus::Validated => {
                match next {
                    BatchedLimitMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            BatchedLimitMetricStatus::Published => {
                return next == BatchedLimitMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: BatchedLimitMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched limit metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "batched limit metric", self.code)
    }

    pub fn net_load(&self) -> f64 {
        ((self.starts_at as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedLimitMetric {
        BatchedLimitMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            amount: self.amount,
            principal: self.principal.clone(),
            revision: self.revision,
            starts_at: self.starts_at,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a batched limit metric in status created.
pub fn new_batched_limit_metric(code: &str, name: &str) -> BatchedLimitMetric {
    BatchedLimitMetric {
        code: code.to_string(),
        name: name.to_string(),
        quantity: 0,
        amount: 0.0,
        principal: "".to_string(),
        revision: 0,
        starts_at: 0,
        expires_at: 0,
        status: BatchedLimitMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_batched_limit_metric("code-660", "name-923");
        assert_eq!(item.status, BatchedLimitMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_batched_limit_metric("code-660", "name-923");
        if item.move_to(BatchedLimitMetricStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedLimitMetricStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_batched_limit_metric("code-660", "name-923");
        item.status = BatchedLimitMetricStatus::Retired;
        assert!(
            item.move_to(BatchedLimitMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
