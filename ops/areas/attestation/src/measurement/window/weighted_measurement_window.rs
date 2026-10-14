use super::*;

use acme_kit::text;

/// Where a weighted measurement window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedMeasurementWindowStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl WeightedMeasurementWindowStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for WeightedMeasurementWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted measurement window. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedMeasurementWindow {
    pub code: String,
    pub title: String,
    pub rotation_days: i64,
    pub key_id: String,
    pub issuer: String,
    pub phone: String,
    pub discount: f64,
    pub status: WeightedMeasurementWindowStatus,
}

impl WeightedMeasurementWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedMeasurementWindowStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedMeasurementWindowStatus) -> bool {
        match self.status {
            WeightedMeasurementWindowStatus::Proposed => {
                match next {
                    WeightedMeasurementWindowStatus::Approved | WeightedMeasurementWindowStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedMeasurementWindowStatus::Approved => {
                return next == WeightedMeasurementWindowStatus::Rejected;
            }
            WeightedMeasurementWindowStatus::Rejected => {
                return next == WeightedMeasurementWindowStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedMeasurementWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted measurement window {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "weighted measurement window", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.discount - (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedMeasurementWindow {
        WeightedMeasurementWindow {
            code: self.code.clone(),
            title: self.title.clone(),
            rotation_days: self.rotation_days,
            key_id: self.key_id.clone(),
            issuer: self.issuer.clone(),
            phone: self.phone.clone(),
            discount: self.discount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a weighted measurement window in status proposed.
pub fn new_weighted_measurement_window(code: &str, title: &str) -> WeightedMeasurementWindow {
    WeightedMeasurementWindow {
        code: code.to_string(),
        title: title.to_string(),
        rotation_days: 0,
        key_id: "".to_string(),
        issuer: "".to_string(),
        phone: "".to_string(),
        discount: 0.0,
        status: WeightedMeasurementWindowStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_weighted_measurement_window("code-933", "title-971");
        assert_eq!(item.status, WeightedMeasurementWindowStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_weighted_measurement_window("code-933", "title-971");
        if item.move_to(WeightedMeasurementWindowStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedMeasurementWindowStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_weighted_measurement_window("code-933", "title-971");
        item.status = WeightedMeasurementWindowStatus::Withdrawn;
        assert!(
            item.move_to(WeightedMeasurementWindowStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
