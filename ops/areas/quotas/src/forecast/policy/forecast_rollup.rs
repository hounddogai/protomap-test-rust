use super::*;

use acme_kit::text;

/// Where a forecast rollup is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastRollupStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl ForecastRollupStatus {
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

impl std::fmt::Display for ForecastRollupStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast rollup as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastRollup {
    pub code: String,
    pub name: String,
    pub cost: f64,
    pub summary: String,
    pub namespace: String,
    pub external_ref: String,
    pub status: ForecastRollupStatus,
}

impl ForecastRollup {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastRollupStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastRollupStatus) -> bool {
        match self.status {
            ForecastRollupStatus::Planned => {
                return next == ForecastRollupStatus::Completed;
            }
            ForecastRollupStatus::Scheduled => {
                match next {
                    ForecastRollupStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastRollupStatus::InProgress => {
                match next {
                    ForecastRollupStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastRollupStatus::Completed => {
                return next == ForecastRollupStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastRollupStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast rollup {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.cost
    }

    pub fn copy(&self) -> ForecastRollup {
        ForecastRollup {
            code: self.code.clone(),
            name: self.name.clone(),
            cost: self.cost,
            summary: self.summary.clone(),
            namespace: self.namespace.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a forecast rollup in status planned.
pub fn new_forecast_rollup(code: &str, name: &str) -> ForecastRollup {
    ForecastRollup {
        code: code.to_string(),
        name: name.to_string(),
        cost: 0.0,
        summary: "".to_string(),
        namespace: "".to_string(),
        external_ref: "".to_string(),
        status: ForecastRollupStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_forecast_rollup("code-638", "name-735");
        assert_eq!(item.status, ForecastRollupStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_forecast_rollup("code-638", "name-735");
        if item.move_to(ForecastRollupStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastRollupStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_forecast_rollup("code-638", "name-735");
        item.status = ForecastRollupStatus::Cancelled;
        assert!(
            item.move_to(ForecastRollupStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
