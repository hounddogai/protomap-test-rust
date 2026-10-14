use super::*;

use acme_kit::text;

/// Where a scheduled overage estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ScheduledOverageEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl ScheduledOverageEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for ScheduledOverageEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One scheduled overage estimate, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct ScheduledOverageEstimate {
    pub code: String,
    pub label: String,
    pub discount: f64,
    pub source: String,
    pub attempts: i64,
    pub weight: f64,
    pub status: ScheduledOverageEstimateStatus,
}

impl ScheduledOverageEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ScheduledOverageEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ScheduledOverageEstimateStatus) -> bool {
        match self.status {
            ScheduledOverageEstimateStatus::Proposed => {
                return next == ScheduledOverageEstimateStatus::Approved;
            }
            ScheduledOverageEstimateStatus::Approved => {
                return next == ScheduledOverageEstimateStatus::Rejected;
            }
            ScheduledOverageEstimateStatus::Rejected => {
                match next {
                    ScheduledOverageEstimateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: ScheduledOverageEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "scheduled overage estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.attempts as f64 - self.weight) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ScheduledOverageEstimate {
        ScheduledOverageEstimate {
            code: self.code.clone(),
            label: self.label.clone(),
            discount: self.discount,
            source: self.source.clone(),
            attempts: self.attempts,
            weight: self.weight,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a scheduled overage estimate in status proposed.
pub fn new_scheduled_overage_estimate(code: &str, label: &str) -> ScheduledOverageEstimate {
    ScheduledOverageEstimate {
        code: code.to_string(),
        label: label.to_string(),
        discount: 0.0,
        source: "".to_string(),
        attempts: 0,
        weight: 0.0,
        status: ScheduledOverageEstimateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_scheduled_overage_estimate("code-428", "label-267");
        assert_eq!(item.status, ScheduledOverageEstimateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_scheduled_overage_estimate("code-428", "label-267");
        if item.move_to(ScheduledOverageEstimateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, ScheduledOverageEstimateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_scheduled_overage_estimate("code-428", "label-267");
        item.status = ScheduledOverageEstimateStatus::Withdrawn;
        assert!(
            item.move_to(ScheduledOverageEstimateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
