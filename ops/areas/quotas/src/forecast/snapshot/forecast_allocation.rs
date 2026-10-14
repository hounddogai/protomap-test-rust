use super::*;

use acme_kit::text;

/// Where a forecast allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastAllocationStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastAllocationStatus {
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

impl std::fmt::Display for ForecastAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastAllocation {
    pub code: String,
    pub name: String,
    pub region: String,
    pub updated_at: i64,
    pub contact_email: String,
    pub country: String,
    pub status: ForecastAllocationStatus,
}

impl ForecastAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastAllocationStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastAllocationStatus) -> bool {
        match self.status {
            ForecastAllocationStatus::Idle => {
                return next == ForecastAllocationStatus::Warming || next == ForecastAllocationStatus::Ready;
            }
            ForecastAllocationStatus::Warming => {
                return next == ForecastAllocationStatus::Draining;
            }
            ForecastAllocationStatus::Ready => {
                return next == ForecastAllocationStatus::Draining;
            }
            ForecastAllocationStatus::Draining => {
                return next == ForecastAllocationStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn measured_value(&self) -> f64 {
        self.updated_at as f64
    }

    pub fn copy(&self) -> ForecastAllocation {
        ForecastAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            region: self.region.clone(),
            updated_at: self.updated_at,
            contact_email: self.contact_email.clone(),
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a forecast allocation in status idle.
pub fn new_forecast_allocation(code: &str, name: &str) -> ForecastAllocation {
    ForecastAllocation {
        code: code.to_string(),
        name: name.to_string(),
        region: "".to_string(),
        updated_at: 0,
        contact_email: "".to_string(),
        country: "".to_string(),
        status: ForecastAllocationStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_forecast_allocation("code-978", "name-787");
        assert_eq!(item.status, ForecastAllocationStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_forecast_allocation("code-978", "name-787");
        if item.move_to(ForecastAllocationStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastAllocationStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_forecast_allocation("code-978", "name-787");
        item.status = ForecastAllocationStatus::Stopped;
        assert!(
            item.move_to(ForecastAllocationStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
