use super::*;

use acme_kit::text;

/// Where a ttl policy threshold is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TtlPolicyThresholdStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl TtlPolicyThresholdStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for TtlPolicyThresholdStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One ttl policy threshold, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct TtlPolicyThreshold {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub maximum: f64,
    pub country: String,
    pub discount: f64,
    pub quota_units: i64,
    pub priority: i64,
    pub rotation_days: i64,
    pub status: TtlPolicyThresholdStatus,
}

impl TtlPolicyThreshold {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TtlPolicyThresholdStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TtlPolicyThresholdStatus) -> bool {
        match self.status {
            TtlPolicyThresholdStatus::Proposed => {
                match next {
                    TtlPolicyThresholdStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyThresholdStatus::Approved => {
                match next {
                    TtlPolicyThresholdStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TtlPolicyThresholdStatus::Rejected => {
                return next == TtlPolicyThresholdStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: TtlPolicyThresholdStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "ttl policy threshold {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "ttl policy threshold", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.maximum + (self.quota_units as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TtlPolicyThreshold {
        TtlPolicyThreshold {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            maximum: self.maximum,
            country: self.country.clone(),
            discount: self.discount,
            quota_units: self.quota_units,
            priority: self.priority,
            rotation_days: self.rotation_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a ttl policy threshold in status proposed.
pub fn new_ttl_policy_threshold(code: &str, label: &str) -> TtlPolicyThreshold {
    TtlPolicyThreshold {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        maximum: 0.0,
        country: "".to_string(),
        discount: 0.0,
        quota_units: 0,
        priority: 0,
        rotation_days: 0,
        status: TtlPolicyThresholdStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_ttl_policy_threshold("code-673", "label-470");
        assert_eq!(item.status, TtlPolicyThresholdStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_ttl_policy_threshold("code-673", "label-470");
        if item.move_to(TtlPolicyThresholdStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, TtlPolicyThresholdStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_ttl_policy_threshold("code-673", "label-470");
        item.status = TtlPolicyThresholdStatus::Withdrawn;
        assert!(
            item.move_to(TtlPolicyThresholdStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
