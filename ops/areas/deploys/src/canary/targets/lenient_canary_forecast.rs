use super::*;

use acme_kit::text;

/// Where a lenient canary forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientCanaryForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl LenientCanaryForecastStatus {
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

impl std::fmt::Display for LenientCanaryForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A lenient canary forecast as the canary team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientCanaryForecast {
    pub code: String,
    pub label: String,
    pub quota_units: i64,
    pub unit: String,
    pub channel: String,
    pub name: String,
    pub external_ref: String,
    pub rotation_days: i64,
    pub status: LenientCanaryForecastStatus,
}

impl LenientCanaryForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientCanaryForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientCanaryForecastStatus) -> bool {
        match self.status {
            LenientCanaryForecastStatus::Idle => {
                return next == LenientCanaryForecastStatus::Warming || next == LenientCanaryForecastStatus::Stopped;
            }
            LenientCanaryForecastStatus::Warming => {
                match next {
                    LenientCanaryForecastStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientCanaryForecastStatus::Ready => {
                return next == LenientCanaryForecastStatus::Draining || next == LenientCanaryForecastStatus::Stopped;
            }
            LenientCanaryForecastStatus::Draining => {
                match next {
                    LenientCanaryForecastStatus::Stopped => {
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

    pub fn move_to(&mut self, next: LenientCanaryForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient canary forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn net_total(&self) -> f64 {
        ((self.rotation_days as f64 - (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientCanaryForecast {
        LenientCanaryForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            quota_units: self.quota_units,
            unit: self.unit.clone(),
            channel: self.channel.clone(),
            name: self.name.clone(),
            external_ref: self.external_ref.clone(),
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a lenient canary forecast in status idle.
pub fn new_lenient_canary_forecast(code: &str, label: &str) -> LenientCanaryForecast {
    LenientCanaryForecast {
        code: code.to_string(),
        label: label.to_string(),
        quota_units: 0,
        unit: "".to_string(),
        channel: "".to_string(),
        name: "".to_string(),
        external_ref: "".to_string(),
        rotation_days: 0,
        status: LenientCanaryForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_lenient_canary_forecast("code-507", "label-191");
        assert_eq!(item.status, LenientCanaryForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_warming() {
        let mut item = new_lenient_canary_forecast("code-507", "label-191");
        if item.move_to(LenientCanaryForecastStatus::Warming).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientCanaryForecastStatus::Warming, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_lenient_canary_forecast("code-507", "label-191");
        item.status = LenientCanaryForecastStatus::Stopped;
        assert!(
            item.move_to(LenientCanaryForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
