use super::*;

use acme_kit::text;

/// Where a primary plan assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPlanAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryPlanAssignmentStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Planned => "planned",
            Self::Scheduled => "scheduled",
            Self::InProgress => "in_progress",
            Self::Completed => "completed",
            Self::Cancelled => "cancelled",
        }
    }
}

impl std::fmt::Display for PrimaryPlanAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a primary plan assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanAssignment {
    pub code: String,
    pub label: String,
    pub namespace: String,
    pub priority: i64,
    pub tags: Vec<String>,
    pub budget: f64,
    pub quota_units: i64,
    pub audience: String,
    pub approved: bool,
    pub channel: String,
    pub retries: i64,
    pub status: PrimaryPlanAssignmentStatus,
}

impl PrimaryPlanAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPlanAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPlanAssignmentStatus) -> bool {
        match self.status {
            PrimaryPlanAssignmentStatus::Planned => {
                match next {
                    PrimaryPlanAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPlanAssignmentStatus::Scheduled => {
                return next == PrimaryPlanAssignmentStatus::Completed
                    || next == PrimaryPlanAssignmentStatus::Cancelled;
            }
            PrimaryPlanAssignmentStatus::InProgress => {
                match next {
                    PrimaryPlanAssignmentStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPlanAssignmentStatus::Completed => {
                return next == PrimaryPlanAssignmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryPlanAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary plan assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_value(&self) -> f64 {
        ((self.priority as f64) * self.budget * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPlanAssignment {
        PrimaryPlanAssignment {
            code: self.code.clone(),
            label: self.label.clone(),
            namespace: self.namespace.clone(),
            priority: self.priority,
            tags: self.tags.clone(),
            budget: self.budget,
            quota_units: self.quota_units,
            audience: self.audience.clone(),
            approved: self.approved,
            channel: self.channel.clone(),
            retries: self.retries,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a primary plan assignment in status planned.
pub fn new_primary_plan_assignment(code: &str, label: &str) -> PrimaryPlanAssignment {
    PrimaryPlanAssignment {
        code: code.to_string(),
        label: label.to_string(),
        namespace: "".to_string(),
        priority: 0,
        tags: Vec::new(),
        budget: 0.0,
        quota_units: 0,
        audience: "".to_string(),
        approved: false,
        channel: "".to_string(),
        retries: 0,
        status: PrimaryPlanAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_plan_assignment("code-383", "label-985");
        assert_eq!(item.status, PrimaryPlanAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_primary_plan_assignment("code-383", "label-985");
        if item.move_to(PrimaryPlanAssignmentStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPlanAssignmentStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_plan_assignment("code-383", "label-985");
        item.status = PrimaryPlanAssignmentStatus::Cancelled;
        assert!(
            item.move_to(PrimaryPlanAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
