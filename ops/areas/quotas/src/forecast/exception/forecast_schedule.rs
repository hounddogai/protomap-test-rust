use super::*;

use acme_kit::text;

/// Where a forecast schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ForecastScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ForecastScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One forecast schedule, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastSchedule {
    pub code: String,
    pub name: String,
    pub window_days: i64,
    pub revision: i64,
    pub region: String,
    pub fingerprint: String,
    pub country: String,
    pub external_ref: String,
    pub minimum: f64,
    pub tenant_id: String,
    pub status: ForecastScheduleStatus,
}

impl ForecastSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastScheduleStatus) -> bool {
        match self.status {
            ForecastScheduleStatus::Requested => {
                match next {
                    ForecastScheduleStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastScheduleStatus::Granted => {
                return next == ForecastScheduleStatus::Revoked || next == ForecastScheduleStatus::Expired;
            }
            ForecastScheduleStatus::Revoked => {
                match next {
                    ForecastScheduleStatus::Expired => {
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

    pub fn move_to(&mut self, next: ForecastScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.revision as f64) * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastSchedule {
        ForecastSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            window_days: self.window_days,
            revision: self.revision,
            region: self.region.clone(),
            fingerprint: self.fingerprint.clone(),
            country: self.country.clone(),
            external_ref: self.external_ref.clone(),
            minimum: self.minimum,
            tenant_id: self.tenant_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a forecast schedule in status requested.
pub fn new_forecast_schedule(code: &str, name: &str) -> ForecastSchedule {
    ForecastSchedule {
        code: code.to_string(),
        name: name.to_string(),
        window_days: 0,
        revision: 0,
        region: "".to_string(),
        fingerprint: "".to_string(),
        country: "".to_string(),
        external_ref: "".to_string(),
        minimum: 0.0,
        tenant_id: "".to_string(),
        status: ForecastScheduleStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_forecast_schedule("code-997", "name-928");
        assert_eq!(item.status, ForecastScheduleStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_forecast_schedule("code-997", "name-928");
        if item.move_to(ForecastScheduleStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastScheduleStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_forecast_schedule("code-997", "name-928");
        item.status = ForecastScheduleStatus::Expired;
        assert!(
            item.move_to(ForecastScheduleStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
