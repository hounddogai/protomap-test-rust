use super::*;

use acme_kit::text;

/// Where a measurement window is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementWindowStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementWindowStatus {
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

impl std::fmt::Display for MeasurementWindowStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement window as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementWindow {
    pub code: String,
    pub label: String,
    pub minimum: f64,
    pub ends_at: i64,
    pub rate: f64,
    pub status: MeasurementWindowStatus,
}

impl MeasurementWindow {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementWindowStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementWindowStatus) -> bool {
        match self.status {
            MeasurementWindowStatus::Idle => {
                match next {
                    MeasurementWindowStatus::Draining | MeasurementWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementWindowStatus::Warming => {
                return next == MeasurementWindowStatus::Draining;
            }
            MeasurementWindowStatus::Ready => {
                match next {
                    MeasurementWindowStatus::Draining | MeasurementWindowStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementWindowStatus::Draining => {
                match next {
                    MeasurementWindowStatus::Stopped => {
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

    pub fn move_to(&mut self, next: MeasurementWindowStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement window {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.rate * (self.ends_at as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> MeasurementWindow {
        MeasurementWindow {
            code: self.code.clone(),
            label: self.label.clone(),
            minimum: self.minimum,
            ends_at: self.ends_at,
            rate: self.rate,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a measurement window in status idle.
pub fn new_measurement_window(code: &str, label: &str) -> MeasurementWindow {
    MeasurementWindow {
        code: code.to_string(),
        label: label.to_string(),
        minimum: 0.0,
        ends_at: 0,
        rate: 0.0,
        status: MeasurementWindowStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_measurement_window("code-697", "label-198");
        assert_eq!(item.status, MeasurementWindowStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_measurement_window("code-697", "label-198");
        if item.move_to(MeasurementWindowStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementWindowStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_measurement_window("code-697", "label-198");
        item.status = MeasurementWindowStatus::Stopped;
        assert!(
            item.move_to(MeasurementWindowStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
