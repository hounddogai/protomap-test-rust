use super::*;

use acme_kit::text;

/// Where a adaptive measurement agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveMeasurementAgreementStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl AdaptiveMeasurementAgreementStatus {
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

impl std::fmt::Display for AdaptiveMeasurementAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive measurement agreement and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveMeasurementAgreement {
    pub code: String,
    pub label: String,
    pub version: i64,
    pub amount: f64,
    pub sequence: i64,
    pub limit: i64,
    pub port: i64,
    pub batch_size: i64,
    pub scope: String,
    pub summary: String,
    pub status: AdaptiveMeasurementAgreementStatus,
}

impl AdaptiveMeasurementAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveMeasurementAgreementStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveMeasurementAgreementStatus) -> bool {
        match self.status {
            AdaptiveMeasurementAgreementStatus::Idle => {
                match next {
                    AdaptiveMeasurementAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMeasurementAgreementStatus::Warming => {
                match next {
                    AdaptiveMeasurementAgreementStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveMeasurementAgreementStatus::Ready => {
                return next == AdaptiveMeasurementAgreementStatus::Draining;
            }
            AdaptiveMeasurementAgreementStatus::Draining => {
                return next == AdaptiveMeasurementAgreementStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveMeasurementAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive measurement agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.version += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.amount * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveMeasurementAgreement {
        AdaptiveMeasurementAgreement {
            code: self.code.clone(),
            label: self.label.clone(),
            version: self.version,
            amount: self.amount,
            sequence: self.sequence,
            limit: self.limit,
            port: self.port,
            batch_size: self.batch_size,
            scope: self.scope.clone(),
            summary: self.summary.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive measurement agreement in status idle.
pub fn new_adaptive_measurement_agreement(code: &str, label: &str) -> AdaptiveMeasurementAgreement {
    AdaptiveMeasurementAgreement {
        code: code.to_string(),
        label: label.to_string(),
        version: 0,
        amount: 0.0,
        sequence: 0,
        limit: 0,
        port: 0,
        batch_size: 0,
        scope: "".to_string(),
        summary: "".to_string(),
        status: AdaptiveMeasurementAgreementStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_adaptive_measurement_agreement("code-268", "label-999");
        assert_eq!(item.status, AdaptiveMeasurementAgreementStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_adaptive_measurement_agreement("code-268", "label-999");
        if item.move_to(AdaptiveMeasurementAgreementStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, AdaptiveMeasurementAgreementStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_adaptive_measurement_agreement("code-268", "label-999");
        item.status = AdaptiveMeasurementAgreementStatus::Stopped;
        assert!(
            item.move_to(AdaptiveMeasurementAgreementStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
