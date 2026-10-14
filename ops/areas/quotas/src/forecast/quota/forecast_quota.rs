use super::*;

use acme_kit::text;

/// Where a forecast quota is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ForecastQuotaStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl ForecastQuotaStatus {
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

impl std::fmt::Display for ForecastQuotaStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A forecast quota. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ForecastQuota {
    pub code: String,
    pub title: String,
    pub quantity: i64,
    pub weight: f64,
    pub channel: String,
    pub discount: f64,
    pub host: String,
    pub active: bool,
    pub retries: i64,
    pub status: ForecastQuotaStatus,
}

impl ForecastQuota {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ForecastQuotaStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ForecastQuotaStatus) -> bool {
        match self.status {
            ForecastQuotaStatus::Idle => {
                return next == ForecastQuotaStatus::Draining;
            }
            ForecastQuotaStatus::Warming => {
                match next {
                    ForecastQuotaStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ForecastQuotaStatus::Ready => {
                return next == ForecastQuotaStatus::Stopped;
            }
            ForecastQuotaStatus::Draining => {
                return next == ForecastQuotaStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ForecastQuotaStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "forecast quota {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.discount + (self.quantity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ForecastQuota {
        ForecastQuota {
            code: self.code.clone(),
            title: self.title.clone(),
            quantity: self.quantity,
            weight: self.weight,
            channel: self.channel.clone(),
            discount: self.discount,
            host: self.host.clone(),
            active: self.active,
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a forecast quota in status idle.
pub fn new_forecast_quota(code: &str, title: &str) -> ForecastQuota {
    ForecastQuota {
        code: code.to_string(),
        title: title.to_string(),
        quantity: 0,
        weight: 0.0,
        channel: "".to_string(),
        discount: 0.0,
        host: "".to_string(),
        active: false,
        retries: 0,
        status: ForecastQuotaStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_forecast_quota("code-887", "title-345");
        assert_eq!(item.status, ForecastQuotaStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_forecast_quota("code-887", "title-345");
        if item.move_to(ForecastQuotaStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ForecastQuotaStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_forecast_quota("code-887", "title-345");
        item.status = ForecastQuotaStatus::Stopped;
        assert!(
            item.move_to(ForecastQuotaStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
