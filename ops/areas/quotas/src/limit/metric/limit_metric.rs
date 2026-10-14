use super::*;

use acme_kit::text;

/// Where a limit metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LimitMetricStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl LimitMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for LimitMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One limit metric, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct LimitMetric {
    pub code: String,
    pub name: String,
    pub audience: String,
    pub batch_size: i64,
    pub cost: f64,
    pub minimum: f64,
    pub status: LimitMetricStatus,
}

impl LimitMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LimitMetricStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LimitMetricStatus) -> bool {
        match self.status {
            LimitMetricStatus::Created => {
                return next == LimitMetricStatus::Validated || next == LimitMetricStatus::Published;
            }
            LimitMetricStatus::Validated => {
                match next {
                    LimitMetricStatus::Published | LimitMetricStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LimitMetricStatus::Published => {
                return next == LimitMetricStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: LimitMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "limit metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "limit metric", self.code)
    }

    pub fn weighted_exposure(&self) -> f64 {
        (self.minimum * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LimitMetric {
        LimitMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            audience: self.audience.clone(),
            batch_size: self.batch_size,
            cost: self.cost,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a limit metric in status created.
pub fn new_limit_metric(code: &str, name: &str) -> LimitMetric {
    LimitMetric {
        code: code.to_string(),
        name: name.to_string(),
        audience: "".to_string(),
        batch_size: 0,
        cost: 0.0,
        minimum: 0.0,
        status: LimitMetricStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_limit_metric("code-837", "name-915");
        assert_eq!(item.status, LimitMetricStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_limit_metric("code-837", "name-915");
        if item.move_to(LimitMetricStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LimitMetricStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_limit_metric("code-837", "name-915");
        item.status = LimitMetricStatus::Retired;
        assert!(
            item.move_to(LimitMetricStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
