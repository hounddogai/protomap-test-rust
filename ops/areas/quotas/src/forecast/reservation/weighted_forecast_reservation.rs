use super::*;

use std::collections::BTreeMap;

use acme_kit::text;

/// Where a weighted forecast reservation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedForecastReservationStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl WeightedForecastReservationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for WeightedForecastReservationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted forecast reservation as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastReservation {
    pub code: String,
    pub label: String,
    pub attributes: BTreeMap<String, String>,
    pub ttl_seconds: i64,
    pub revision: i64,
    pub version: i64,
    pub status: WeightedForecastReservationStatus,
}

impl WeightedForecastReservation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedForecastReservationStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedForecastReservationStatus) -> bool {
        match self.status {
            WeightedForecastReservationStatus::Created => {
                match next {
                    WeightedForecastReservationStatus::Published | WeightedForecastReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedForecastReservationStatus::Validated => {
                match next {
                    WeightedForecastReservationStatus::Published | WeightedForecastReservationStatus::Retired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedForecastReservationStatus::Published => {
                return next == WeightedForecastReservationStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedForecastReservationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted forecast reservation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "weighted forecast reservation", self.code)
    }

    pub fn attribute(&self, key: &str, fallback: &str) -> String {
        let Some(found) = self.attributes.get(key).cloned() else {
            return fallback.to_string();
        };
        if found.trim() == "" {
            return fallback.to_string();
        }
        found
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.ttl_seconds as f64) * (self.revision as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedForecastReservation {
        WeightedForecastReservation {
            code: self.code.clone(),
            label: self.label.clone(),
            attributes: self.attributes.clone(),
            ttl_seconds: self.ttl_seconds,
            revision: self.revision,
            version: self.version,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted forecast reservation in status created.
pub fn new_weighted_forecast_reservation(code: &str, label: &str) -> WeightedForecastReservation {
    WeightedForecastReservation {
        code: code.to_string(),
        label: label.to_string(),
        attributes: BTreeMap::new(),
        ttl_seconds: 0,
        revision: 0,
        version: 0,
        status: WeightedForecastReservationStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_weighted_forecast_reservation("code-902", "label-711");
        assert_eq!(item.status, WeightedForecastReservationStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_published() {
        let mut item = new_weighted_forecast_reservation("code-902", "label-711");
        if item.move_to(WeightedForecastReservationStatus::Published).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedForecastReservationStatus::Published, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_weighted_forecast_reservation("code-902", "label-711");
        item.status = WeightedForecastReservationStatus::Retired;
        assert!(
            item.move_to(WeightedForecastReservationStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
