use super::*;

use acme_kit::text;

/// Where a rolling quote estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingQuoteEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl RollingQuoteEstimateStatus {
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

impl std::fmt::Display for RollingQuoteEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a rolling quote estimate and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingQuoteEstimate {
    pub code: String,
    pub label: String,
    pub attempts: i64,
    pub maximum: f64,
    pub checksum: String,
    pub tenant_id: String,
    pub category: String,
    pub active: bool,
    pub country: String,
    pub external_ref: String,
    pub status: RollingQuoteEstimateStatus,
}

impl RollingQuoteEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingQuoteEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingQuoteEstimateStatus) -> bool {
        match self.status {
            RollingQuoteEstimateStatus::Idle => {
                match next {
                    RollingQuoteEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingQuoteEstimateStatus::Warming => {
                return next == RollingQuoteEstimateStatus::Ready;
            }
            RollingQuoteEstimateStatus::Ready => {
                match next {
                    RollingQuoteEstimateStatus::Draining => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RollingQuoteEstimateStatus::Draining => {
                return next == RollingQuoteEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingQuoteEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling quote estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "rolling quote estimate", self.code)
    }

    pub fn effective_total(&self) -> f64 {
        ((self.attempts as f64 - self.maximum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingQuoteEstimate {
        RollingQuoteEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            attempts: self.attempts,
            maximum: self.maximum,
            checksum: self.checksum.clone(),
            tenant_id: self.tenant_id.clone(),
            category: self.category.clone(),
            active: self.active,
            country: self.country.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling quote estimate in status idle.
pub fn new_rolling_quote_estimate(code: &str, label: &str) -> RollingQuoteEstimate {
    RollingQuoteEstimate {
        code: code.to_string(),
        label: label.to_string(),
        attempts: 0,
        maximum: 0.0,
        checksum: "".to_string(),
        tenant_id: "".to_string(),
        category: "".to_string(),
        active: false,
        country: "".to_string(),
        external_ref: "".to_string(),
        status: RollingQuoteEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_rolling_quote_estimate("code-118", "label-23");
        assert_eq!(item.status, RollingQuoteEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_rolling_quote_estimate("code-118", "label-23");
        if item.move_to(RollingQuoteEstimateStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RollingQuoteEstimateStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_rolling_quote_estimate("code-118", "label-23");
        item.status = RollingQuoteEstimateStatus::Stopped;
        assert!(
            item.move_to(RollingQuoteEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
