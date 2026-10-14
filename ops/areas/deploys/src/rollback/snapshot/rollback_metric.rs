use super::*;

use acme_kit::text;

/// Where a rollback metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollbackMetricStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollbackMetricStatus {
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

impl std::fmt::Display for RollbackMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rollback metric as the rollback team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollbackMetric {
    pub code: String,
    pub name: String,
    pub scope: String,
    pub country: String,
    pub unit: String,
    pub burst: i64,
    pub status: RollbackMetricStatus,
}

impl RollbackMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollbackMetricStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollbackMetricStatus) -> bool {
        match self.status {
            RollbackMetricStatus::Draft => {
                return next == RollbackMetricStatus::Pending || next == RollbackMetricStatus::Closed;
            }
            RollbackMetricStatus::Pending => {
                return next == RollbackMetricStatus::Active || next == RollbackMetricStatus::Suspended;
            }
            RollbackMetricStatus::Active => {
                return next == RollbackMetricStatus::Closed;
            }
            RollbackMetricStatus::Suspended => {
                return next == RollbackMetricStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollbackMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rollback metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.burst as f64
    }

    pub fn copy(&self) -> RollbackMetric {
        RollbackMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            scope: self.scope.clone(),
            country: self.country.clone(),
            unit: self.unit.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a rollback metric in status draft.
pub fn new_rollback_metric(code: &str, name: &str) -> RollbackMetric {
    RollbackMetric {
        code: code.to_string(),
        name: name.to_string(),
        scope: "".to_string(),
        country: "".to_string(),
        unit: "".to_string(),
        burst: 0,
        status: RollbackMetricStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_rollback_metric("code-728", "name-516");
        assert_eq!(item.status, RollbackMetricStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_pending() {
        let mut item = new_rollback_metric("code-728", "name-516");
        if item.move_to(RollbackMetricStatus::Pending).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollbackMetricStatus::Pending, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_rollback_metric("code-728", "name-516");
        item.status = RollbackMetricStatus::Closed;
        assert!(
            item.move_to(RollbackMetricStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
