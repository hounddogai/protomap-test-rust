use super::*;

use acme_kit::text;

/// Where a weighted forecast metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedForecastMetricStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedForecastMetricStatus {
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

impl std::fmt::Display for WeightedForecastMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted forecast metric as the forecast team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedForecastMetric {
    pub code: String,
    pub name: String,
    pub segment: String,
    pub principal: String,
    pub source: String,
    pub key_id: String,
    pub cipher: String,
    pub rotation_days: i64,
    pub namespace: String,
    pub capacity: i64,
    pub status: WeightedForecastMetricStatus,
}

impl WeightedForecastMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedForecastMetricStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedForecastMetricStatus) -> bool {
        match self.status {
            WeightedForecastMetricStatus::Idle => {
                return next == WeightedForecastMetricStatus::Warming || next == WeightedForecastMetricStatus::Draining;
            }
            WeightedForecastMetricStatus::Warming => {
                match next {
                    WeightedForecastMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedForecastMetricStatus::Ready => {
                match next {
                    WeightedForecastMetricStatus::Draining | WeightedForecastMetricStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedForecastMetricStatus::Draining => {
                match next {
                    WeightedForecastMetricStatus::Stopped => {
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

    pub fn move_to(&mut self, next: WeightedForecastMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted forecast metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.rotation_days as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedForecastMetric {
        WeightedForecastMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            segment: self.segment.clone(),
            principal: self.principal.clone(),
            source: self.source.clone(),
            key_id: self.key_id.clone(),
            cipher: self.cipher.clone(),
            rotation_days: self.rotation_days,
            namespace: self.namespace.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a weighted forecast metric in status idle.
pub fn new_weighted_forecast_metric(code: &str, name: &str) -> WeightedForecastMetric {
    WeightedForecastMetric {
        code: code.to_string(),
        name: name.to_string(),
        segment: "".to_string(),
        principal: "".to_string(),
        source: "".to_string(),
        key_id: "".to_string(),
        cipher: "".to_string(),
        rotation_days: 0,
        namespace: "".to_string(),
        capacity: 0,
        status: WeightedForecastMetricStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_forecast_metric("code-986", "name-626");
        assert_eq!(item.status, WeightedForecastMetricStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_weighted_forecast_metric("code-986", "name-626");
        if item.move_to(WeightedForecastMetricStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedForecastMetricStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_forecast_metric("code-986", "name-626");
        item.status = WeightedForecastMetricStatus::Stopped;
        assert!(
            item.move_to(WeightedForecastMetricStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
