use super::*;

use acme_kit::text;

/// Where a health check budget is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum HealthCheckBudgetStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl HealthCheckBudgetStatus {
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

impl std::fmt::Display for HealthCheckBudgetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One health check budget, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct HealthCheckBudget {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub namespace: String,
    pub version: i64,
    pub cost: f64,
    pub reviewer: String,
    pub ttl_seconds: i64,
    pub locale: String,
    pub burst: i64,
    pub status: HealthCheckBudgetStatus,
}

impl HealthCheckBudget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            HealthCheckBudgetStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: HealthCheckBudgetStatus) -> bool {
        match self.status {
            HealthCheckBudgetStatus::Draft => {
                match next {
                    HealthCheckBudgetStatus::Closed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            HealthCheckBudgetStatus::Pending => {
                return next == HealthCheckBudgetStatus::Closed;
            }
            HealthCheckBudgetStatus::Active => {
                return next == HealthCheckBudgetStatus::Suspended;
            }
            HealthCheckBudgetStatus::Suspended => {
                match next {
                    HealthCheckBudgetStatus::Closed => {
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

    pub fn move_to(&mut self, next: HealthCheckBudgetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "health check budget {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.version as f64 + self.margin) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> HealthCheckBudget {
        HealthCheckBudget {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            namespace: self.namespace.clone(),
            version: self.version,
            cost: self.cost,
            reviewer: self.reviewer.clone(),
            ttl_seconds: self.ttl_seconds,
            locale: self.locale.clone(),
            burst: self.burst,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a health check budget in status draft.
pub fn new_health_check_budget(code: &str, label: &str) -> HealthCheckBudget {
    HealthCheckBudget {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        namespace: "".to_string(),
        version: 0,
        cost: 0.0,
        reviewer: "".to_string(),
        ttl_seconds: 0,
        locale: "".to_string(),
        burst: 0,
        status: HealthCheckBudgetStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_health_check_budget("code-470", "label-339");
        assert_eq!(item.status, HealthCheckBudgetStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_closed() {
        let mut item = new_health_check_budget("code-470", "label-339");
        if item.move_to(HealthCheckBudgetStatus::Closed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, HealthCheckBudgetStatus::Closed, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_health_check_budget("code-470", "label-339");
        item.status = HealthCheckBudgetStatus::Closed;
        assert!(
            item.move_to(HealthCheckBudgetStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
