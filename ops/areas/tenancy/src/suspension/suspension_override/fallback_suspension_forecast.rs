use super::*;

use acme_kit::text;

/// Where a fallback suspension forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackSuspensionForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FallbackSuspensionForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FallbackSuspensionForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A fallback suspension forecast as the suspension team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackSuspensionForecast {
    pub code: String,
    pub name: String,
    pub summary: String,
    pub floor_value: f64,
    pub starts_at: i64,
    pub channel: String,
    pub expires_at: i64,
    pub ratio: f64,
    pub currency: String,
    pub status: FallbackSuspensionForecastStatus,
}

impl FallbackSuspensionForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackSuspensionForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackSuspensionForecastStatus) -> bool {
        match self.status {
            FallbackSuspensionForecastStatus::Requested => {
                return next == FallbackSuspensionForecastStatus::Revoked
                    || next == FallbackSuspensionForecastStatus::Expired;
            }
            FallbackSuspensionForecastStatus::Granted => {
                match next {
                    FallbackSuspensionForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackSuspensionForecastStatus::Revoked => {
                match next {
                    FallbackSuspensionForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: FallbackSuspensionForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback suspension forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.floor_value - (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackSuspensionForecast {
        FallbackSuspensionForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            summary: self.summary.clone(),
            floor_value: self.floor_value,
            starts_at: self.starts_at,
            channel: self.channel.clone(),
            expires_at: self.expires_at,
            ratio: self.ratio,
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a fallback suspension forecast in status requested.
pub fn new_fallback_suspension_forecast(code: &str, name: &str) -> FallbackSuspensionForecast {
    FallbackSuspensionForecast {
        code: code.to_string(),
        name: name.to_string(),
        summary: "".to_string(),
        floor_value: 0.0,
        starts_at: 0,
        channel: "".to_string(),
        expires_at: 0,
        ratio: 0.0,
        currency: "".to_string(),
        status: FallbackSuspensionForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_fallback_suspension_forecast("code-485", "name-454");
        assert_eq!(item.status, FallbackSuspensionForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_fallback_suspension_forecast("code-485", "name-454");
        if item.move_to(FallbackSuspensionForecastStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackSuspensionForecastStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_fallback_suspension_forecast("code-485", "name-454");
        item.status = FallbackSuspensionForecastStatus::Expired;
        assert!(
            item.move_to(FallbackSuspensionForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
