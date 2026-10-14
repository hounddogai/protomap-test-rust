use super::*;

use acme_kit::text;

/// Where a ttl policy forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl TtlPolicyForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for TtlPolicyForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyForecast {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub unit: String,
    pub floor_value: f64,
    pub pinned: bool,
    pub retries: i64,
    pub visible: bool,
    pub quantity: i64,
    pub status: TtlPolicyForecastStatus,
}

impl TtlPolicyForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyForecastStatus) -> bool {
        match self.status {
            TtlPolicyForecastStatus::Requested => {
                match next {
                    TtlPolicyForecastStatus::Granted => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyForecastStatus::Granted => {
                return next == TtlPolicyForecastStatus::Revoked;
            }
            TtlPolicyForecastStatus::Revoked => {
                return next == TtlPolicyForecastStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "ttl policy forecast", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.floor_value + (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyForecast {
        TtlPolicyForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            unit: self.unit.clone(),
            floor_value: self.floor_value,
            pinned: self.pinned,
            retries: self.retries,
            visible: self.visible,
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a ttl policy forecast in status requested.
pub fn new_ttl_policy_forecast(code: &str, label: &str) -> TtlPolicyForecast {
    TtlPolicyForecast {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        unit: "".to_string(),
        floor_value: 0.0,
        pinned: false,
        retries: 0,
        visible: false,
        quantity: 0,
        status: TtlPolicyForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_ttl_policy_forecast("code-66", "label-863");
        assert_eq!(item.status, TtlPolicyForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_ttl_policy_forecast("code-66", "label-863");
        if item.move_to(TtlPolicyForecastStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyForecastStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_ttl_policy_forecast("code-66", "label-863");
        item.status = TtlPolicyForecastStatus::Expired;
        assert!(
            item.move_to(TtlPolicyForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
