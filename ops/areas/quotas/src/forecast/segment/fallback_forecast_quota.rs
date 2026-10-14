use super::*;

use acme_kit::text;

/// Where a fallback forecast quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackForecastQuotaStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackForecastQuotaStatus {
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

impl std::fmt::Display for FallbackForecastQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback forecast quota, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackForecastQuota {
    pub code: String,
    pub title: String,
    pub issuer: String,
    pub host: String,
    pub source: String,
    pub ttl_seconds: i64,
    pub audience: String,
    pub duration_minutes: i64,
    pub status: FallbackForecastQuotaStatus,
}

impl FallbackForecastQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackForecastQuotaStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackForecastQuotaStatus) -> bool {
        match self.status {
            FallbackForecastQuotaStatus::Planned => {
                match next {
                    FallbackForecastQuotaStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackForecastQuotaStatus::Scheduled => {
                return next == FallbackForecastQuotaStatus::InProgress
                    || next == FallbackForecastQuotaStatus::Completed;
            }
            FallbackForecastQuotaStatus::InProgress => {
                return next == FallbackForecastQuotaStatus::Completed
                    || next == FallbackForecastQuotaStatus::Cancelled;
            }
            FallbackForecastQuotaStatus::Completed => {
                return next == FallbackForecastQuotaStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackForecastQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback forecast quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn net_value(&self) -> f64 {
        ((self.duration_minutes as f64 + (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackForecastQuota {
        FallbackForecastQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            issuer: self.issuer.clone(),
            host: self.host.clone(),
            source: self.source.clone(),
            ttl_seconds: self.ttl_seconds,
            audience: self.audience.clone(),
            duration_minutes: self.duration_minutes,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a fallback forecast quota in status planned.
pub fn new_fallback_forecast_quota(code: &str, title: &str) -> FallbackForecastQuota {
    FallbackForecastQuota {
        code: code.to_string(),
        title: title.to_string(),
        issuer: "".to_string(),
        host: "".to_string(),
        source: "".to_string(),
        ttl_seconds: 0,
        audience: "".to_string(),
        duration_minutes: 0,
        status: FallbackForecastQuotaStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_forecast_quota("code-621", "title-480");
        assert_eq!(item.status, FallbackForecastQuotaStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_fallback_forecast_quota("code-621", "title-480");
        if item.move_to(FallbackForecastQuotaStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackForecastQuotaStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_forecast_quota("code-621", "title-480");
        item.status = FallbackForecastQuotaStatus::Cancelled;
        assert!(
            item.move_to(FallbackForecastQuotaStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
