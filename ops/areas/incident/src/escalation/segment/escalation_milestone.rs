use super::*;

use acme_kit::text;

/// Where a escalation milestone is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationMilestoneStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EscalationMilestoneStatus {
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

impl std::fmt::Display for EscalationMilestoneStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A escalation milestone. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMilestone {
    pub code: String,
    pub name: String,
    pub country: String,
    pub tags: Vec<String>,
    pub budget: f64,
    pub score: f64,
    pub visible: bool,
    pub pinned: bool,
    pub status: EscalationMilestoneStatus,
}

impl EscalationMilestone {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationMilestoneStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationMilestoneStatus) -> bool {
        match self.status {
            EscalationMilestoneStatus::Planned => {
                match next {
                    EscalationMilestoneStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationMilestoneStatus::Scheduled => {
                match next {
                    EscalationMilestoneStatus::InProgress | EscalationMilestoneStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationMilestoneStatus::InProgress => {
                return next == EscalationMilestoneStatus::Completed;
            }
            EscalationMilestoneStatus::Completed => {
                return next == EscalationMilestoneStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: EscalationMilestoneStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation milestone {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn effective_load(&self) -> f64 {
        (self.budget * self.score * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationMilestone {
        EscalationMilestone {
            code: self.code.clone(),
            name: self.name.clone(),
            country: self.country.clone(),
            tags: self.tags.clone(),
            budget: self.budget,
            score: self.score,
            visible: self.visible,
            pinned: self.pinned,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a escalation milestone in status planned.
pub fn new_escalation_milestone(code: &str, name: &str) -> EscalationMilestone {
    EscalationMilestone {
        code: code.to_string(),
        name: name.to_string(),
        country: "".to_string(),
        tags: Vec::new(),
        budget: 0.0,
        score: 0.0,
        visible: false,
        pinned: false,
        status: EscalationMilestoneStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_escalation_milestone("code-591", "name-447");
        assert_eq!(item.status, EscalationMilestoneStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_escalation_milestone("code-591", "name-447");
        if item.move_to(EscalationMilestoneStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationMilestoneStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_escalation_milestone("code-591", "name-447");
        item.status = EscalationMilestoneStatus::Cancelled;
        assert!(
            item.move_to(EscalationMilestoneStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
