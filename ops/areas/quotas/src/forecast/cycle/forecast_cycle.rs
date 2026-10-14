use super::*;

use acme_kit::text;

/// Where a forecast cycle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastCycleStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl ForecastCycleStatus {
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

impl std::fmt::Display for ForecastCycleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast cycle and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastCycle {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub limit: i64,
    pub title: String,
    pub scope: String,
    pub ratio: f64,
    pub minimum: f64,
    pub status: ForecastCycleStatus,
}

impl ForecastCycle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastCycleStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastCycleStatus) -> bool {
        match self.status {
            ForecastCycleStatus::Draft => {
                match next {
                    ForecastCycleStatus::Active => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastCycleStatus::Pending => {
                return next == ForecastCycleStatus::Active || next == ForecastCycleStatus::Closed;
            }
            ForecastCycleStatus::Active => {
                return next == ForecastCycleStatus::Suspended;
            }
            ForecastCycleStatus::Suspended => {
                match next {
                    ForecastCycleStatus::Closed => {
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

    pub fn move_to(&mut self, next: ForecastCycleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast cycle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "forecast cycle", self.code)
    }

    pub fn net_load(&self) -> f64 {
        (self.ratio * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastCycle {
        ForecastCycle {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            limit: self.limit,
            title: self.title.clone(),
            scope: self.scope.clone(),
            ratio: self.ratio,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a forecast cycle in status draft.
pub fn new_forecast_cycle(code: &str, label: &str) -> ForecastCycle {
    ForecastCycle {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        limit: 0,
        title: "".to_string(),
        scope: "".to_string(),
        ratio: 0.0,
        minimum: 0.0,
        status: ForecastCycleStatus::Draft,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_draft() {
        let item = new_forecast_cycle("code-393", "label-984");
        assert_eq!(item.status, ForecastCycleStatus::Draft, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_active() {
        let mut item = new_forecast_cycle("code-393", "label-984");
        if item.move_to(ForecastCycleStatus::Active).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastCycleStatus::Active, "status");
    }

    #[test]
    fn rejects_moves_from_closed() {
        let mut item = new_forecast_cycle("code-393", "label-984");
        item.status = ForecastCycleStatus::Closed;
        assert!(
            item.move_to(ForecastCycleStatus::Draft).is_err(),
            "expected move to fail"
        );
    }
}
