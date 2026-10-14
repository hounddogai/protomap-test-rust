use super::*;

use acme_kit::text;

/// Where a weighted policy estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedPolicyEstimateStatus {
    #[default]
    Idle,
    Warming,
    Ready,
    Draining,
    Stopped,
}

impl WeightedPolicyEstimateStatus {
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

impl std::fmt::Display for WeightedPolicyEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted policy estimate as the policy team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedPolicyEstimate {
    pub code: String,
    pub label: String,
    pub key_id: String,
    pub ttl_seconds: i64,
    pub external_ref: String,
    pub sequence: i64,
    pub limit: i64,
    pub rate: f64,
    pub audience: String,
    pub status: WeightedPolicyEstimateStatus,
}

impl WeightedPolicyEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedPolicyEstimateStatus::Stopped => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedPolicyEstimateStatus) -> bool {
        match self.status {
            WeightedPolicyEstimateStatus::Idle => {
                match next {
                    WeightedPolicyEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPolicyEstimateStatus::Warming => {
                match next {
                    WeightedPolicyEstimateStatus::Stopped => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedPolicyEstimateStatus::Ready => {
                return next == WeightedPolicyEstimateStatus::Draining;
            }
            WeightedPolicyEstimateStatus::Draining => {
                return next == WeightedPolicyEstimateStatus::Stopped;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: WeightedPolicyEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted policy estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        (self.rate * (self.sequence as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedPolicyEstimate {
        WeightedPolicyEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            key_id: self.key_id.clone(),
            ttl_seconds: self.ttl_seconds,
            external_ref: self.external_ref.clone(),
            sequence: self.sequence,
            limit: self.limit,
            rate: self.rate,
            audience: self.audience.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a weighted policy estimate in status idle.
pub fn new_weighted_policy_estimate(code: &str, label: &str) -> WeightedPolicyEstimate {
    WeightedPolicyEstimate {
        code: code.to_string(),
        label: label.to_string(),
        key_id: "".to_string(),
        ttl_seconds: 0,
        external_ref: "".to_string(),
        sequence: 0,
        limit: 0,
        rate: 0.0,
        audience: "".to_string(),
        status: WeightedPolicyEstimateStatus::Idle,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_idle() {
        let item = new_weighted_policy_estimate("code-442", "label-498");
        assert_eq!(item.status, WeightedPolicyEstimateStatus::Idle, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_stopped() {
        let mut item = new_weighted_policy_estimate("code-442", "label-498");
        if item.move_to(WeightedPolicyEstimateStatus::Stopped).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, WeightedPolicyEstimateStatus::Stopped, "status");
    }

    #[test]
    fn rejects_moves_from_stopped() {
        let mut item = new_weighted_policy_estimate("code-442", "label-498");
        item.status = WeightedPolicyEstimateStatus::Stopped;
        assert!(
            item.move_to(WeightedPolicyEstimateStatus::Idle).is_err(),
            "expected move to fail"
        );
    }
}
