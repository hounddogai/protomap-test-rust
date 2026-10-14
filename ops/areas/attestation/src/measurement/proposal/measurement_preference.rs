use super::*;

use acme_kit::text;

/// Where a measurement preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementPreferenceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementPreferenceStatus {
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

impl std::fmt::Display for MeasurementPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement preference. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementPreference {
    pub code: String,
    pub title: String,
    pub minimum: f64,
    pub batch_size: i64,
    pub phone: String,
    pub description: String,
    pub channel: String,
    pub status: MeasurementPreferenceStatus,
}

impl MeasurementPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementPreferenceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementPreferenceStatus) -> bool {
        match self.status {
            MeasurementPreferenceStatus::Idle => {
                match next {
                    MeasurementPreferenceStatus::Warming | MeasurementPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementPreferenceStatus::Warming => {
                return next == MeasurementPreferenceStatus::Ready || next == MeasurementPreferenceStatus::Stopped;
            }
            MeasurementPreferenceStatus::Ready => {
                match next {
                    MeasurementPreferenceStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementPreferenceStatus::Draining => {
                return next == MeasurementPreferenceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.minimum - (self.batch_size as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementPreference {
        MeasurementPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            minimum: self.minimum,
            batch_size: self.batch_size,
            phone: self.phone.clone(),
            description: self.description.clone(),
            channel: self.channel.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a measurement preference in status idle.
pub fn new_measurement_preference(code: &str, title: &str) -> MeasurementPreference {
    MeasurementPreference {
        code: code.to_string(),
        title: title.to_string(),
        minimum: 0.0,
        batch_size: 0,
        phone: "".to_string(),
        description: "".to_string(),
        channel: "".to_string(),
        status: MeasurementPreferenceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_measurement_preference("code-378", "title-222");
        assert_eq!(item.status, MeasurementPreferenceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_measurement_preference("code-378", "title-222");
        if item.move_to(MeasurementPreferenceStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementPreferenceStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_measurement_preference("code-378", "title-222");
        item.status = MeasurementPreferenceStatus::Stopped;
        assert!(
            item.move_to(MeasurementPreferenceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
