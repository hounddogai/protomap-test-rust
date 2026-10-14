use super::*;

use acme_kit::text;

/// Where a weighted measurement metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedMeasurementMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedMeasurementMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedMeasurementMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted measurement metric. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementMetric {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub budget: f64,
    pub capacity: i64,
    pub rate: f64,
    pub batch_size: i64,
    pub expires_at: i64,
    pub owner_id: String,
    pub summary: String,
    pub status: WeightedMeasurementMetricStatus,
}

impl WeightedMeasurementMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedMeasurementMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedMeasurementMetricStatus) -> bool {
        match self.status {
            WeightedMeasurementMetricStatus::Proposed => {
                return next == WeightedMeasurementMetricStatus::Approved
                    || next == WeightedMeasurementMetricStatus::Withdrawn;
            }
            WeightedMeasurementMetricStatus::Approved => {
                return next == WeightedMeasurementMetricStatus::Withdrawn;
            }
            WeightedMeasurementMetricStatus::Rejected => {
                match next {
                    WeightedMeasurementMetricStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: WeightedMeasurementMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted measurement metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.rate - (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedMeasurementMetric {
        WeightedMeasurementMetric {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            budget: self.budget,
            capacity: self.capacity,
            rate: self.rate,
            batch_size: self.batch_size,
            expires_at: self.expires_at,
            owner_id: self.owner_id.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a weighted measurement metric in status proposed.
pub fn new_weighted_measurement_metric(code: &str, name: &str) -> WeightedMeasurementMetric {
    WeightedMeasurementMetric {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        budget: 0.0,
        capacity: 0,
        rate: 0.0,
        batch_size: 0,
        expires_at: 0,
        owner_id: "".to_string(),
        summary: "".to_string(),
        status: WeightedMeasurementMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_measurement_metric("code-120", "name-20");
        assert_eq!(item.status, WeightedMeasurementMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_weighted_measurement_metric("code-120", "name-20");
        if item.move_to(WeightedMeasurementMetricStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedMeasurementMetricStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_measurement_metric("code-120", "name-20");
        item.status = WeightedMeasurementMetricStatus::Withdrawn;
        assert!(
            item.move_to(WeightedMeasurementMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
