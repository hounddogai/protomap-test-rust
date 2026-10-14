use super::*;

use acme_kit::text;

/// Where a overage forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum OverageForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl OverageForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for OverageForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A overage forecast as the overage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct OverageForecast {
    pub code: String,
    pub label: String,
    pub name: String,
    pub host: String,
    pub duration_minutes: i64,
    pub version: i64,
    pub cipher: String,
    pub expires_at: i64,
    pub audience: String,
    pub quota_units: i64,
    pub region: String,
    pub status: OverageForecastStatus,
}

impl OverageForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            OverageForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: OverageForecastStatus) -> bool {
        match self.status {
            OverageForecastStatus::Requested => {
                match next {
                    OverageForecastStatus::Granted | OverageForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageForecastStatus::Granted => {
                match next {
                    OverageForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            OverageForecastStatus::Revoked => {
                match next {
                    OverageForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: OverageForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "overage forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "overage forecast", self.code)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.expires_at as f64) * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> OverageForecast {
        OverageForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            name: self.name.clone(),
            host: self.host.clone(),
            duration_minutes: self.duration_minutes,
            version: self.version,
            cipher: self.cipher.clone(),
            expires_at: self.expires_at,
            audience: self.audience.clone(),
            quota_units: self.quota_units,
            region: self.region.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a overage forecast in status requested.
pub fn new_overage_forecast(code: &str, label: &str) -> OverageForecast {
    OverageForecast {
        code: code.to_string(),
        label: label.to_string(),
        name: "".to_string(),
        host: "".to_string(),
        duration_minutes: 0,
        version: 0,
        cipher: "".to_string(),
        expires_at: 0,
        audience: "".to_string(),
        quota_units: 0,
        region: "".to_string(),
        status: OverageForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_overage_forecast("code-805", "label-939");
        assert_eq!(item.status, OverageForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_overage_forecast("code-805", "label-939");
        if item.move_to(OverageForecastStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, OverageForecastStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_overage_forecast("code-805", "label-939");
        item.status = OverageForecastStatus::Expired;
        assert!(
            item.move_to(OverageForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
