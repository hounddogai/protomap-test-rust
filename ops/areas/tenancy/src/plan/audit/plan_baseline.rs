use super::*;

use acme_kit::text;

/// Where a plan baseline is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanBaselineStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanBaselineStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanBaselineStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan baseline. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanBaseline {
    pub code: String,
    pub label: String,
    pub sequence: i64,
    pub rotation_days: i64,
    pub budget: f64,
    pub scope: String,
    pub status: PlanBaselineStatus,
}

impl PlanBaseline {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanBaselineStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanBaselineStatus) -> bool {
        match self.status {
            PlanBaselineStatus::Proposed => {
                return next == PlanBaselineStatus::Approved || next == PlanBaselineStatus::Rejected;
            }
            PlanBaselineStatus::Approved => {
                return next == PlanBaselineStatus::Rejected || next == PlanBaselineStatus::Withdrawn;
            }
            PlanBaselineStatus::Rejected => {
                return next == PlanBaselineStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanBaselineStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan baseline {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn adjusted_exposure(&self) -> f64 {
        ((self.sequence as f64 - self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanBaseline {
        PlanBaseline {
            code: self.code.clone(),
            label: self.label.clone(),
            sequence: self.sequence,
            rotation_days: self.rotation_days,
            budget: self.budget,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a plan baseline in status proposed.
pub fn new_plan_baseline(code: &str, label: &str) -> PlanBaseline {
    PlanBaseline {
        code: code.to_string(),
        label: label.to_string(),
        sequence: 0,
        rotation_days: 0,
        budget: 0.0,
        scope: "".to_string(),
        status: PlanBaselineStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_plan_baseline("code-641", "label-267");
        assert_eq!(item.status, PlanBaselineStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_plan_baseline("code-641", "label-267");
        if item.move_to(PlanBaselineStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanBaselineStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_plan_baseline("code-641", "label-267");
        item.status = PlanBaselineStatus::Withdrawn;
        assert!(
            item.move_to(PlanBaselineStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
