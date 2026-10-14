use super::*;

use acme_kit::text;

/// Where a fallback delegation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackDelegationForecastStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackDelegationForecastStatus {
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

impl std::fmt::Display for FallbackDelegationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback delegation forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackDelegationForecast {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub title: String,
    pub ratio: f64,
    pub created_at: i64,
    pub duration_minutes: i64,
    pub minimum: f64,
    pub status: FallbackDelegationForecastStatus,
}

impl FallbackDelegationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackDelegationForecastStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackDelegationForecastStatus) -> bool {
        match self.status {
            FallbackDelegationForecastStatus::Planned => {
                match next {
                    FallbackDelegationForecastStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDelegationForecastStatus::Scheduled => {
                return next == FallbackDelegationForecastStatus::Completed
                    || next == FallbackDelegationForecastStatus::Cancelled;
            }
            FallbackDelegationForecastStatus::InProgress => {
                match next {
                    FallbackDelegationForecastStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackDelegationForecastStatus::Completed => {
                return next == FallbackDelegationForecastStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackDelegationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback delegation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        (self.ratio * self.minimum * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackDelegationForecast {
        FallbackDelegationForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            title: self.title.clone(),
            ratio: self.ratio,
            created_at: self.created_at,
            duration_minutes: self.duration_minutes,
            minimum: self.minimum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a fallback delegation forecast in status planned.
pub fn new_fallback_delegation_forecast(code: &str, name: &str) -> FallbackDelegationForecast {
    FallbackDelegationForecast {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        title: "".to_string(),
        ratio: 0.0,
        created_at: 0,
        duration_minutes: 0,
        minimum: 0.0,
        status: FallbackDelegationForecastStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_delegation_forecast("code-619", "name-454");
        assert_eq!(item.status, FallbackDelegationForecastStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_fallback_delegation_forecast("code-619", "name-454");
        if item.move_to(FallbackDelegationForecastStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackDelegationForecastStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_delegation_forecast("code-619", "name-454");
        item.status = FallbackDelegationForecastStatus::Cancelled;
        assert!(
            item.move_to(FallbackDelegationForecastStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
