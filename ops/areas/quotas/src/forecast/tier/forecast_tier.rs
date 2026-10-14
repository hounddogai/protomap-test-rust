use super::*;

use acme_kit::text;

/// Where a forecast tier is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastTierStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastTierStatus {
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

impl std::fmt::Display for ForecastTierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a forecast tier and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastTier {
    pub code: String,
    pub name: String,
    pub country: String,
    pub contact_email: String,
    pub audience: String,
    pub amount: f64,
    pub status: ForecastTierStatus,
}

impl ForecastTier {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastTierStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastTierStatus) -> bool {
        match self.status {
            ForecastTierStatus::Idle => {
                return next == ForecastTierStatus::Ready;
            }
            ForecastTierStatus::Warming => {
                match next {
                    ForecastTierStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastTierStatus::Ready => {
                match next {
                    ForecastTierStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastTierStatus::Draining => {
                return next == ForecastTierStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastTierStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast tier {} cannot move from {} to {}",
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
        self.amount
    }

    pub fn copy(&self) -> ForecastTier {
        ForecastTier {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            contact_email: self.contact_email.clone(),
            audience: self.audience.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a forecast tier in status idle.
pub fn new_forecast_tier(code: &str, name: &str) -> ForecastTier {
    ForecastTier {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        contact_email: "".to_string(),
        audience: "".to_string(),
        amount: 0.0,
        status: ForecastTierStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_forecast_tier("code-177", "name-392");
        assert_eq!(item.status, ForecastTierStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_forecast_tier("code-177", "name-392");
        if item.move_to(ForecastTierStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastTierStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_forecast_tier("code-177", "name-392");
        item.status = ForecastTierStatus::Stopped;
        assert!(item.move_to(ForecastTierStatus::Idle).is_err(), "expected move to fail");
    }
}
