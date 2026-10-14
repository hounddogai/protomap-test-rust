use super::*;

use acme_kit::text;

/// Where a default measurement override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DefaultMeasurementOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl DefaultMeasurementOverrideStatus {
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

impl std::fmt::Display for DefaultMeasurementOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a default measurement override and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DefaultMeasurementOverride {
    pub code: String,
    pub label: String,
    pub scope: String,
    pub cost: f64,
    pub expires_at: i64,
    pub reviewer: String,
    pub description: String,
    pub status: DefaultMeasurementOverrideStatus,
}

impl DefaultMeasurementOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DefaultMeasurementOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DefaultMeasurementOverrideStatus) -> bool {
        match self.status {
            DefaultMeasurementOverrideStatus::Idle => {
                match next {
                    DefaultMeasurementOverrideStatus::Warming | DefaultMeasurementOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMeasurementOverrideStatus::Warming => {
                match next {
                    DefaultMeasurementOverrideStatus::Ready | DefaultMeasurementOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DefaultMeasurementOverrideStatus::Ready => {
                return next == DefaultMeasurementOverrideStatus::Stopped;
            }
            DefaultMeasurementOverrideStatus::Draining => {
                return next == DefaultMeasurementOverrideStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DefaultMeasurementOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "default measurement override {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn weighted_volume(&self) -> f64 {
        ((self.cost + (self.expires_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DefaultMeasurementOverride {
        DefaultMeasurementOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            scope: self.scope.clone(),
            cost: self.cost,
            expires_at: self.expires_at,
            reviewer: self.reviewer.clone(),
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a default measurement override in status idle.
pub fn new_default_measurement_override(code: &str, label: &str) -> DefaultMeasurementOverride {
    DefaultMeasurementOverride {
        code: code.to_string(),
        label: label.to_string(),
        scope: "".to_string(),
        cost: 0.0,
        expires_at: 0,
        reviewer: "".to_string(),
        description: "".to_string(),
        status: DefaultMeasurementOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_default_measurement_override("code-100", "label-624");
        assert_eq!(item.status, DefaultMeasurementOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_default_measurement_override("code-100", "label-624");
        if item.move_to(DefaultMeasurementOverrideStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, DefaultMeasurementOverrideStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_default_measurement_override("code-100", "label-624");
        item.status = DefaultMeasurementOverrideStatus::Stopped;
        assert!(
            item.move_to(DefaultMeasurementOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
