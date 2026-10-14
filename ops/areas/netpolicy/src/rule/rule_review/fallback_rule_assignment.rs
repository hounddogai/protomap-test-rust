use super::*;

use acme_kit::text;

/// Where a fallback rule assignment is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackRuleAssignmentStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl FallbackRuleAssignmentStatus {
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

impl std::fmt::Display for FallbackRuleAssignmentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a fallback rule assignment and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackRuleAssignment {
    pub code: String,
    pub title: String,
    pub name: String,
    pub quantity: i64,
    pub limit: i64,
    pub visible: bool,
    pub maximum: f64,
    pub status: FallbackRuleAssignmentStatus,
}

impl FallbackRuleAssignment {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackRuleAssignmentStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackRuleAssignmentStatus) -> bool {
        match self.status {
            FallbackRuleAssignmentStatus::Planned => {
                match next {
                    FallbackRuleAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRuleAssignmentStatus::Scheduled => {
                match next {
                    FallbackRuleAssignmentStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRuleAssignmentStatus::InProgress => {
                match next {
                    FallbackRuleAssignmentStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackRuleAssignmentStatus::Completed => {
                return next == FallbackRuleAssignmentStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FallbackRuleAssignmentStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback rule assignment {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn effective_exposure(&self) -> f64 {
        (self.maximum * (self.limit as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackRuleAssignment {
        FallbackRuleAssignment {
            code: self.code.clone(),
            title: self.title.clone(),
            name: self.name.clone(),
            quantity: self.quantity,
            limit: self.limit,
            visible: self.visible,
            maximum: self.maximum,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a fallback rule assignment in status planned.
pub fn new_fallback_rule_assignment(code: &str, title: &str) -> FallbackRuleAssignment {
    FallbackRuleAssignment {
        code: code.to_string(),
        title: title.to_string(),
        name: "".to_string(),
        quantity: 0,
        limit: 0,
        visible: false,
        maximum: 0.0,
        status: FallbackRuleAssignmentStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_fallback_rule_assignment("code-51", "title-13");
        assert_eq!(item.status, FallbackRuleAssignmentStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_cancelled() {
        let mut item = new_fallback_rule_assignment("code-51", "title-13");
        if item.move_to(FallbackRuleAssignmentStatus::Cancelled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackRuleAssignmentStatus::Cancelled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_fallback_rule_assignment("code-51", "title-13");
        item.status = FallbackRuleAssignmentStatus::Cancelled;
        assert!(
            item.move_to(FallbackRuleAssignmentStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
