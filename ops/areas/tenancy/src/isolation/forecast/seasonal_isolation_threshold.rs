use super::*;

use acme_kit::text;

/// Where a seasonal isolation threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SeasonalIsolationThresholdStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl SeasonalIsolationThresholdStatus {
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

impl std::fmt::Display for SeasonalIsolationThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One seasonal isolation threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct SeasonalIsolationThreshold {
    pub code: String,
    pub name: String,
    pub unit: String,
    pub revision: i64,
    pub amount: f64,
    pub status: SeasonalIsolationThresholdStatus,
}

impl SeasonalIsolationThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            SeasonalIsolationThresholdStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: SeasonalIsolationThresholdStatus) -> bool {
        match self.status {
            SeasonalIsolationThresholdStatus::Idle => {
                match next {
                    SeasonalIsolationThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIsolationThresholdStatus::Warming => {
                return next == SeasonalIsolationThresholdStatus::Draining;
            }
            SeasonalIsolationThresholdStatus::Ready => {
                match next {
                    SeasonalIsolationThresholdStatus::Draining | SeasonalIsolationThresholdStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            SeasonalIsolationThresholdStatus::Draining => {
                match next {
                    SeasonalIsolationThresholdStatus::Stopped => {
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

    pub fn move_to(&mut self, next: SeasonalIsolationThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "seasonal isolation threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.revision as f64 - self.amount) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> SeasonalIsolationThreshold {
        SeasonalIsolationThreshold {
            code: self.code.clone(),
            name: self.name.clone(),
            unit: self.unit.clone(),
            revision: self.revision,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a seasonal isolation threshold in status idle.
pub fn new_seasonal_isolation_threshold(code: &str, name: &str) -> SeasonalIsolationThreshold {
    SeasonalIsolationThreshold {
        code: code.to_string(),
        name: name.to_string(),
        unit: "".to_string(),
        revision: 0,
        amount: 0.0,
        status: SeasonalIsolationThresholdStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_seasonal_isolation_threshold("code-272", "name-452");
        assert_eq!(item.status, SeasonalIsolationThresholdStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_seasonal_isolation_threshold("code-272", "name-452");
        if item.move_to(SeasonalIsolationThresholdStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, SeasonalIsolationThresholdStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_seasonal_isolation_threshold("code-272", "name-452");
        item.status = SeasonalIsolationThresholdStatus::Stopped;
        assert!(
            item.move_to(SeasonalIsolationThresholdStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
