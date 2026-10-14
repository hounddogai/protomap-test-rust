use super::*;

use acme_kit::text;

/// Where a default measurement mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMeasurementMappingStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultMeasurementMappingStatus {
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

impl std::fmt::Display for DefaultMeasurementMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default measurement mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementMapping {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub updated_at: i64,
    pub currency: String,
    pub grace_days: i64,
    pub maximum: f64,
    pub weight: f64,
    pub status: DefaultMeasurementMappingStatus,
}

impl DefaultMeasurementMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMeasurementMappingStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMeasurementMappingStatus) -> bool {
        match self.status {
            DefaultMeasurementMappingStatus::Idle => {
                return next == DefaultMeasurementMappingStatus::Ready;
            }
            DefaultMeasurementMappingStatus::Warming => {
                match next {
                    DefaultMeasurementMappingStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMeasurementMappingStatus::Ready => {
                return next == DefaultMeasurementMappingStatus::Draining
                    || next == DefaultMeasurementMappingStatus::Stopped;
            }
            DefaultMeasurementMappingStatus::Draining => {
                return next == DefaultMeasurementMappingStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultMeasurementMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default measurement mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn adjusted_total(&self) -> f64 {
        (self.weight * (self.grace_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMeasurementMapping {
        DefaultMeasurementMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            updated_at: self.updated_at,
            currency: self.currency.clone(),
            grace_days: self.grace_days,
            maximum: self.maximum,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a default measurement mapping in status idle.
pub fn new_default_measurement_mapping(code: &str, name: &str) -> DefaultMeasurementMapping {
    DefaultMeasurementMapping {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        updated_at: 0,
        currency: "".to_string(),
        grace_days: 0,
        maximum: 0.0,
        weight: 0.0,
        status: DefaultMeasurementMappingStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_measurement_mapping("code-854", "name-208");
        assert_eq!(item.status, DefaultMeasurementMappingStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_ready() {
        let mut item = new_default_measurement_mapping("code-854", "name-208");
        if item.move_to(DefaultMeasurementMappingStatus::Ready).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultMeasurementMappingStatus::Ready, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_measurement_mapping("code-854", "name-208");
        item.status = DefaultMeasurementMappingStatus::Stopped;
        assert!(
            item.move_to(DefaultMeasurementMappingStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
