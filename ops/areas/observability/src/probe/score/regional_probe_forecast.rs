use super::*;

use acme_kit::text;

/// Where a regional probe forecast is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionalProbeForecastStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RegionalProbeForecastStatus {
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

impl std::fmt::Display for RegionalProbeForecastStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One regional probe forecast, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionalProbeForecast {
    pub code: String,
    pub label: String,
    pub tenant_id: String,
    pub score: f64,
    pub scope: String,
    pub status: RegionalProbeForecastStatus,
}

impl RegionalProbeForecast {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionalProbeForecastStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionalProbeForecastStatus) -> bool {
        match self.status {
            RegionalProbeForecastStatus::Idle => {
                return next == RegionalProbeForecastStatus::Draining || next == RegionalProbeForecastStatus::Stopped;
            }
            RegionalProbeForecastStatus::Warming => {
                return next == RegionalProbeForecastStatus::Ready || next == RegionalProbeForecastStatus::Draining;
            }
            RegionalProbeForecastStatus::Ready => {
                match next {
                    RegionalProbeForecastStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RegionalProbeForecastStatus::Draining => {
                match next {
                    RegionalProbeForecastStatus::Stopped => {
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

    pub fn move_to(&mut self, next: RegionalProbeForecastStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "regional probe forecast {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> RegionalProbeForecast {
        RegionalProbeForecast {
            code: self.code.clone(),
            label: self.label.clone(),
            tenant_id: self.tenant_id.clone(),
            score: self.score,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a regional probe forecast in status idle.
pub fn new_regional_probe_forecast(code: &str, label: &str) -> RegionalProbeForecast {
    RegionalProbeForecast {
        code: code.to_string(),
        label: label.to_string(),
        tenant_id: "".to_string(),
        score: 0.0,
        scope: "".to_string(),
        status: RegionalProbeForecastStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_regional_probe_forecast("code-788", "label-549");
        assert_eq!(item.status, RegionalProbeForecastStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_draining() {
        let mut item = new_regional_probe_forecast("code-788", "label-549");
        if item.move_to(RegionalProbeForecastStatus::Draining).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionalProbeForecastStatus::Draining, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_regional_probe_forecast("code-788", "label-549");
        item.status = RegionalProbeForecastStatus::Stopped;
        assert!(
            item.move_to(RegionalProbeForecastStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
