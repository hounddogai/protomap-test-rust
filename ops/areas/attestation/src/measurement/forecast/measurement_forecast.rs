use super::*;

use acme_kit::text;

/// Where a measurement forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementForecastStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl MeasurementForecastStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for MeasurementForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement forecast. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementForecast {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub notes: String,
    pub sequence: i64,
    pub ends_at: i64,
    pub country: String,
    pub maximum: f64,
    pub status: MeasurementForecastStatus,
}

impl MeasurementForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementForecastStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementForecastStatus) -> bool {
        match self.status {
            MeasurementForecastStatus::Requested => {
                return next == MeasurementForecastStatus::Revoked;
            }
            MeasurementForecastStatus::Granted => {
                match next {
                    MeasurementForecastStatus::Revoked | MeasurementForecastStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementForecastStatus::Revoked => {
                match next {
                    MeasurementForecastStatus::Expired => {
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

    pub fn move_to(&mut self, next: MeasurementForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.ends_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn effective_value(&self) -> f64 {
        ((self.sequence as f64 - (self.ends_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementForecast {
        MeasurementForecast {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            notes: self.notes.clone(),
            sequence: self.sequence,
            ends_at: self.ends_at,
            country: self.country.clone(),
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a measurement forecast in status requested.
pub fn new_measurement_forecast(code: &str, name: &str) -> MeasurementForecast {
    MeasurementForecast {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        notes: "".to_string(),
        sequence: 0,
        ends_at: 0,
        country: "".to_string(),
        maximum: 0.0,
        status: MeasurementForecastStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_measurement_forecast("code-413", "name-306");
        assert_eq!(item.status, MeasurementForecastStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_revoked() {
        let mut item = new_measurement_forecast("code-413", "name-306");
        if item.move_to(MeasurementForecastStatus::Revoked).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementForecastStatus::Revoked, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_measurement_forecast("code-413", "name-306");
        item.status = MeasurementForecastStatus::Expired;
        assert!(
            item.move_to(MeasurementForecastStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
