use super::*;

use acme_kit::text;

/// Where a global measurement override is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalMeasurementOverrideStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl GlobalMeasurementOverrideStatus {
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

impl std::fmt::Display for GlobalMeasurementOverrideStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global measurement override, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalMeasurementOverride {
    pub code: String,
    pub label: String,
    pub port: i64,
    pub locale: String,
    pub ttl_seconds: i64,
    pub status: GlobalMeasurementOverrideStatus,
}

impl GlobalMeasurementOverride {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalMeasurementOverrideStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalMeasurementOverrideStatus) -> bool {
        match self.status {
            GlobalMeasurementOverrideStatus::Idle => {
                match next {
                    GlobalMeasurementOverrideStatus::Warming | GlobalMeasurementOverrideStatus::Ready => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalMeasurementOverrideStatus::Warming => {
                return next == GlobalMeasurementOverrideStatus::Ready
                    || next == GlobalMeasurementOverrideStatus::Draining;
            }
            GlobalMeasurementOverrideStatus::Ready => {
                match next {
                    GlobalMeasurementOverrideStatus::Draining | GlobalMeasurementOverrideStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalMeasurementOverrideStatus::Draining => {
                match next {
                    GlobalMeasurementOverrideStatus::Stopped => {
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

    pub fn move_to(&mut self, next: GlobalMeasurementOverrideStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global measurement override {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalMeasurementOverride {
        GlobalMeasurementOverride {
            code: self.code.clone(),
            label: self.label.clone(),
            port: self.port,
            locale: self.locale.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a global measurement override in status idle.
pub fn new_global_measurement_override(code: &str, label: &str) -> GlobalMeasurementOverride {
    GlobalMeasurementOverride {
        code: code.to_string(),
        label: label.to_string(),
        port: 0,
        locale: "".to_string(),
        ttl_seconds: 0,
        status: GlobalMeasurementOverrideStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_global_measurement_override("code-324", "label-127");
        assert_eq!(item.status, GlobalMeasurementOverrideStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_global_measurement_override("code-324", "label-127");
        if item.move_to(GlobalMeasurementOverrideStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalMeasurementOverrideStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_global_measurement_override("code-324", "label-127");
        item.status = GlobalMeasurementOverrideStatus::Stopped;
        assert!(
            item.move_to(GlobalMeasurementOverrideStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
