use super::*;

use acme_kit::text;

/// Where a federation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FederationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for FederationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One federation forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationForecast {
    pub code: String,
    pub title: String,
    pub capacity: i64,
    pub quota_units: i64,
    pub currency: String,
    pub region: String,
    pub principal: String,
    pub discount: f64,
    pub contact_email: String,
    pub status: FederationForecastStatus,
}

impl FederationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationForecastStatus) -> bool {
        match self.status {
            FederationForecastStatus::New => {
                return next == FederationForecastStatus::Running;
            }
            FederationForecastStatus::Queued => {
                return next == FederationForecastStatus::Failed;
            }
            FederationForecastStatus::Running => {
                return next == FederationForecastStatus::Done;
            }
            FederationForecastStatus::Done => {
                match next {
                    FederationForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: FederationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "federation forecast", self.code)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.discount + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationForecast {
        FederationForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            capacity: self.capacity,
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            region: self.region.clone(),
            principal: self.principal.clone(),
            discount: self.discount,
            contact_email: self.contact_email.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a federation forecast in status new.
pub fn new_federation_forecast(code: &str, title: &str) -> FederationForecast {
    FederationForecast {
        code: code.to_string(),
        title: title.to_string(),
        capacity: 0,
        quota_units: 0,
        currency: "".to_string(),
        region: "".to_string(),
        principal: "".to_string(),
        discount: 0.0,
        contact_email: "".to_string(),
        status: FederationForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_federation_forecast("code-280", "title-731");
        assert_eq!(item.status, FederationForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_running() {
        let mut item = new_federation_forecast("code-280", "title-731");
        if item.move_to(FederationForecastStatus::Running).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FederationForecastStatus::Running, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_federation_forecast("code-280", "title-731");
        item.status = FederationForecastStatus::Failed;
        assert!(
            item.move_to(FederationForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
