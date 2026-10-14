use super::*;

use acme_kit::text;

/// Where a legacy isolation forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyIsolationForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl LegacyIsolationForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for LegacyIsolationForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy isolation forecast as the isolation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIsolationForecast {
    pub code: String,
    pub title: String,
    pub cluster: String,
    pub rate: f64,
    pub limit: i64,
    pub key_id: String,
    pub status: LegacyIsolationForecastStatus,
}

impl LegacyIsolationForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyIsolationForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyIsolationForecastStatus) -> bool {
        match self.status {
            LegacyIsolationForecastStatus::Requested => {
                match next {
                    LegacyIsolationForecastStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIsolationForecastStatus::Granted => {
                match next {
                    LegacyIsolationForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIsolationForecastStatus::Revoked => {
                match next {
                    LegacyIsolationForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: LegacyIsolationForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy isolation forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy isolation forecast", self.code)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.rate - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyIsolationForecast {
        LegacyIsolationForecast {
            code: self.code.clone(),
            title: self.title.clone(),
            cluster: self.cluster.clone(),
            rate: self.rate,
            limit: self.limit,
            key_id: self.key_id.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a legacy isolation forecast in status requested.
pub fn new_legacy_isolation_forecast(code: &str, title: &str) -> LegacyIsolationForecast {
    LegacyIsolationForecast {
        code: code.to_string(),
        title: title.to_string(),
        cluster: "".to_string(),
        rate: 0.0,
        limit: 0,
        key_id: "".to_string(),
        status: LegacyIsolationForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_legacy_isolation_forecast("code-885", "title-418");
        assert_eq!(item.status, LegacyIsolationForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_legacy_isolation_forecast("code-885", "title-418");
        if item.move_to(LegacyIsolationForecastStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyIsolationForecastStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_legacy_isolation_forecast("code-885", "title-418");
        item.status = LegacyIsolationForecastStatus::Expired;
        assert!(
            item.move_to(LegacyIsolationForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
