use super::*;

use acme_kit::text;

/// Where a forecast bundle is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastBundleStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastBundleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Warming => "warming",
            Self::Ready => "ready",
            Self::Draining => "draining",
            Self::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ForecastBundleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast bundle as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastBundle {
    pub code: String,
    pub label: String,
    pub segment: String,
    pub category: String,
    pub unit: String,
    pub sequence: i64,
    pub country: String,
    pub cost: f64,
    pub status: ForecastBundleStatus,
}

impl ForecastBundle {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastBundleStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastBundleStatus) -> bool {
        match self.status {
            ForecastBundleStatus::Idle => {
                return next == ForecastBundleStatus::Ready;
            }
            ForecastBundleStatus::Warming => {
                match next {
                    ForecastBundleStatus::Ready | ForecastBundleStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastBundleStatus::Ready => {
                return next == ForecastBundleStatus::Stopped;
            }
            ForecastBundleStatus::Draining => {
                match next {
                    ForecastBundleStatus::Stopped => {
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

    pub fn move_to(&mut self, next: ForecastBundleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast bundle {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "forecast bundle", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.cost - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastBundle {
        ForecastBundle {
            code: self.code.clone(),
            label: self.label.clone(),
            segment: self.segment.clone(),
            category: self.category.clone(),
            unit: self.unit.clone(),
            sequence: self.sequence,
            country: self.country.clone(),
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a forecast bundle in status idle.
pub fn new_forecast_bundle(code: &str, label: &str) -> ForecastBundle {
    ForecastBundle {
        code: code.to_string(),
        label: label.to_string(),
        segment: "".to_string(),
        category: "".to_string(),
        unit: "".to_string(),
        sequence: 0,
        country: "".to_string(),
        cost: 0.0,
        status: ForecastBundleStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_forecast_bundle("code-528", "label-386");
        assert_eq!(item.status, ForecastBundleStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_forecast_bundle("code-528", "label-386");
        if item.move_to(ForecastBundleStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastBundleStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_forecast_bundle("code-528", "label-386");
        item.status = ForecastBundleStatus::Stopped;
        assert!(
            item.move_to(ForecastBundleStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
