use super::*;

use acme_kit::text;

/// Where a measurement variance is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum MeasurementVarianceStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl MeasurementVarianceStatus {
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

impl std::fmt::Display for MeasurementVarianceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A measurement variance as the measurement team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasurementVariance {
    pub code: String,
    pub label: String,
    pub audience: String,
    pub issuer: String,
    pub currency: String,
    pub score: f64,
    pub status: MeasurementVarianceStatus,
}

impl MeasurementVariance {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            MeasurementVarianceStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: MeasurementVarianceStatus) -> bool {
        match self.status {
            MeasurementVarianceStatus::Idle => {
                return next == MeasurementVarianceStatus::Stopped;
            }
            MeasurementVarianceStatus::Warming => {
                match next {
                    MeasurementVarianceStatus::Ready | MeasurementVarianceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementVarianceStatus::Ready => {
                match next {
                    MeasurementVarianceStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            MeasurementVarianceStatus::Draining => {
                return next == MeasurementVarianceStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: MeasurementVarianceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "measurement variance {} cannot move from {} to {}",
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

    pub fn copy(&self) -> MeasurementVariance {
        MeasurementVariance {
            code: self.code.clone(),
            label: self.label.clone(),
            audience: self.audience.clone(),
            issuer: self.issuer.clone(),
            currency: self.currency.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a measurement variance in status idle.
pub fn new_measurement_variance(code: &str, label: &str) -> MeasurementVariance {
    MeasurementVariance {
        code: code.to_string(),
        label: label.to_string(),
        audience: "".to_string(),
        issuer: "".to_string(),
        currency: "".to_string(),
        score: 0.0,
        status: MeasurementVarianceStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_measurement_variance("code-774", "label-850");
        assert_eq!(item.status, MeasurementVarianceStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_measurement_variance("code-774", "label-850");
        if item.move_to(MeasurementVarianceStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, MeasurementVarianceStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_measurement_variance("code-774", "label-850");
        item.status = MeasurementVarianceStatus::Stopped;
        assert!(
            item.move_to(MeasurementVarianceStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
