use super::*;

use acme_kit::text;

/// Where a lenient policy metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientPolicyMetricStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl LenientPolicyMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for LenientPolicyMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient policy metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientPolicyMetric {
    pub code: String,
    pub title: String,
    pub pinned: bool,
    pub channel: String,
    pub amount: f64,
    pub status: LenientPolicyMetricStatus,
}

impl LenientPolicyMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientPolicyMetricStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientPolicyMetricStatus) -> bool {
        match self.status {
            LenientPolicyMetricStatus::Proposed => {
                return next == LenientPolicyMetricStatus::Approved || next == LenientPolicyMetricStatus::Rejected;
            }
            LenientPolicyMetricStatus::Approved => {
                match next {
                    LenientPolicyMetricStatus::Rejected | LenientPolicyMetricStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientPolicyMetricStatus::Rejected => {
                match next {
                    LenientPolicyMetricStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: LenientPolicyMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient policy metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "lenient policy metric", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.amount
    }

    pub fn copy(&self) -> LenientPolicyMetric {
        LenientPolicyMetric {
            code: self.code.clone(),
            title: self.title.clone(),
            pinned: self.pinned,
            channel: self.channel.clone(),
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.channel)
    }
}

/// Starts a lenient policy metric in status proposed.
pub fn new_lenient_policy_metric(code: &str, title: &str) -> LenientPolicyMetric {
    LenientPolicyMetric {
        code: code.to_string(),
        title: title.to_string(),
        pinned: false,
        channel: "".to_string(),
        amount: 0.0,
        status: LenientPolicyMetricStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_lenient_policy_metric("code-15", "title-402");
        assert_eq!(item.status, LenientPolicyMetricStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_lenient_policy_metric("code-15", "title-402");
        if item.move_to(LenientPolicyMetricStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientPolicyMetricStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_lenient_policy_metric("code-15", "title-402");
        item.status = LenientPolicyMetricStatus::Withdrawn;
        assert!(
            item.move_to(LenientPolicyMetricStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
