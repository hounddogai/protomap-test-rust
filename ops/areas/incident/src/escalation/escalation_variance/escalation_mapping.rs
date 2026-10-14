use super::*;

use acme_kit::text;

/// Where a escalation mapping is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationMappingStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl EscalationMappingStatus {
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

impl std::fmt::Display for EscalationMappingStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation mapping and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationMapping {
    pub code: String,
    pub name: String,
    pub archived: bool,
    pub currency: String,
    pub priority: i64,
    pub window_days: i64,
    pub status: EscalationMappingStatus,
}

impl EscalationMapping {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationMappingStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationMappingStatus) -> bool {
        match self.status {
            EscalationMappingStatus::Open => {
                return next == EscalationMappingStatus::Resolved || next == EscalationMappingStatus::Reopened;
            }
            EscalationMappingStatus::InReview => {
                match next {
                    EscalationMappingStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationMappingStatus::Resolved => {
                match next {
                    EscalationMappingStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationMappingStatus::Reopened => {
                match next {
                    EscalationMappingStatus::Archived => {
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

    pub fn move_to(&mut self, next: EscalationMappingStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation mapping {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "escalation mapping", self.code)
    }

    pub fn adjusted_load(&self) -> f64 {
        ((self.window_days as f64 - (self.priority as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> EscalationMapping {
        EscalationMapping {
            code: self.code.clone(),
            name: self.name.clone(),
            archived: self.archived,
            currency: self.currency.clone(),
            priority: self.priority,
            window_days: self.window_days,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a escalation mapping in status open.
pub fn new_escalation_mapping(code: &str, name: &str) -> EscalationMapping {
    EscalationMapping {
        code: code.to_string(),
        name: name.to_string(),
        archived: false,
        currency: "".to_string(),
        priority: 0,
        window_days: 0,
        status: EscalationMappingStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_escalation_mapping("code-70", "name-448");
        assert_eq!(item.status, EscalationMappingStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_resolved() {
        let mut item = new_escalation_mapping("code-70", "name-448");
        if item.move_to(EscalationMappingStatus::Resolved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationMappingStatus::Resolved, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_escalation_mapping("code-70", "name-448");
        item.status = EscalationMappingStatus::Archived;
        assert!(
            item.move_to(EscalationMappingStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
