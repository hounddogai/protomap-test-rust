use super::*;

use acme_kit::text;

/// Where a fallback peering forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPeeringForecastStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl FallbackPeeringForecastStatus {
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

impl std::fmt::Display for FallbackPeeringForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback peering forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPeeringForecast {
    pub code: String,
    pub name: String,
    pub external_ref: String,
    pub priority: i64,
    pub channel: String,
    pub issuer: String,
    pub retries: i64,
    pub rate: f64,
    pub status: FallbackPeeringForecastStatus,
}

impl FallbackPeeringForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPeeringForecastStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPeeringForecastStatus) -> bool {
        match self.status {
            FallbackPeeringForecastStatus::New => {
                return next == FallbackPeeringForecastStatus::Queued || next == FallbackPeeringForecastStatus::Running;
            }
            FallbackPeeringForecastStatus::Queued => {
                return next == FallbackPeeringForecastStatus::Done;
            }
            FallbackPeeringForecastStatus::Running => {
                match next {
                    FallbackPeeringForecastStatus::Done | FallbackPeeringForecastStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPeeringForecastStatus::Done => {
                match next {
                    FallbackPeeringForecastStatus::Failed => {
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

    pub fn move_to(&mut self, next: FallbackPeeringForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback peering forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.rate + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPeeringForecast {
        FallbackPeeringForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            priority: self.priority,
            channel: self.channel.clone(),
            issuer: self.issuer.clone(),
            retries: self.retries,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback peering forecast in status new.
pub fn new_fallback_peering_forecast(code: &str, name: &str) -> FallbackPeeringForecast {
    FallbackPeeringForecast {
        code: code.to_string(),
        name: name.to_string(),
        external_ref: "".to_string(),
        priority: 0,
        channel: "".to_string(),
        issuer: "".to_string(),
        retries: 0,
        rate: 0.0,
        status: FallbackPeeringForecastStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_fallback_peering_forecast("code-987", "name-237");
        assert_eq!(item.status, FallbackPeeringForecastStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_fallback_peering_forecast("code-987", "name-237");
        if item.move_to(FallbackPeeringForecastStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPeeringForecastStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_fallback_peering_forecast("code-987", "name-237");
        item.status = FallbackPeeringForecastStatus::Failed;
        assert!(
            item.move_to(FallbackPeeringForecastStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
