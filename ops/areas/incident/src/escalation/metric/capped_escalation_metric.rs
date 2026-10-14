use super::*;

use acme_kit::text;

/// Where a capped escalation metric is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum CappedEscalationMetricStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl CappedEscalationMetricStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for CappedEscalationMetricStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a capped escalation metric and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct CappedEscalationMetric {
    pub code: String,
    pub label: String,
    pub rate: f64,
    pub approved: bool,
    pub visible: bool,
    pub floor_value: f64,
    pub budget: f64,
    pub duration_minutes: i64,
    pub cluster: String,
    pub status: CappedEscalationMetricStatus,
}

impl CappedEscalationMetric {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            CappedEscalationMetricStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: CappedEscalationMetricStatus) -> bool {
        match self.status {
            CappedEscalationMetricStatus::Open => {
                return next == CappedEscalationMetricStatus::Resolved
                    || next == CappedEscalationMetricStatus::Archived;
            }
            CappedEscalationMetricStatus::InReview => {
                return next == CappedEscalationMetricStatus::Resolved
                    || next == CappedEscalationMetricStatus::Archived;
            }
            CappedEscalationMetricStatus::Resolved => {
                match next {
                    CappedEscalationMetricStatus::Reopened | CappedEscalationMetricStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            CappedEscalationMetricStatus::Reopened => {
                match next {
                    CappedEscalationMetricStatus::Archived => {
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

    pub fn move_to(&mut self, next: CappedEscalationMetricStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "capped escalation metric {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        (self.rate * (self.duration_minutes as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> CappedEscalationMetric {
        CappedEscalationMetric {
            code: self.code.clone(),
            label: self.label.clone(),
            rate: self.rate,
            approved: self.approved,
            visible: self.visible,
            floor_value: self.floor_value,
            budget: self.budget,
            duration_minutes: self.duration_minutes,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a capped escalation metric in status open.
pub fn new_capped_escalation_metric(code: &str, label: &str) -> CappedEscalationMetric {
    CappedEscalationMetric {
        code: code.to_string(),
        label: label.to_string(),
        rate: 0.0,
        approved: false,
        visible: false,
        floor_value: 0.0,
        budget: 0.0,
        duration_minutes: 0,
        cluster: "".to_string(),
        status: CappedEscalationMetricStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_capped_escalation_metric("code-747", "label-81");
        assert_eq!(item.status, CappedEscalationMetricStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_capped_escalation_metric("code-747", "label-81");
        if item.move_to(CappedEscalationMetricStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, CappedEscalationMetricStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_capped_escalation_metric("code-747", "label-81");
        item.status = CappedEscalationMetricStatus::Archived;
        assert!(
            item.move_to(CappedEscalationMetricStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
