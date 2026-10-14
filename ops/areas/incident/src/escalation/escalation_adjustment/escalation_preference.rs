use super::*;

use acme_kit::text;

/// Where a escalation preference is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum EscalationPreferenceStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl EscalationPreferenceStatus {
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

impl std::fmt::Display for EscalationPreferenceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a escalation preference and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct EscalationPreference {
    pub code: String,
    pub title: String,
    pub category: String,
    pub segment: String,
    pub visible: bool,
    pub score: f64,
    pub status: EscalationPreferenceStatus,
}

impl EscalationPreference {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            EscalationPreferenceStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: EscalationPreferenceStatus) -> bool {
        match self.status {
            EscalationPreferenceStatus::Planned => {
                match next {
                    EscalationPreferenceStatus::Scheduled | EscalationPreferenceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationPreferenceStatus::Scheduled => {
                return next == EscalationPreferenceStatus::InProgress || next == EscalationPreferenceStatus::Completed;
            }
            EscalationPreferenceStatus::InProgress => {
                match next {
                    EscalationPreferenceStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            EscalationPreferenceStatus::Completed => {
                match next {
                    EscalationPreferenceStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: EscalationPreferenceStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "escalation preference {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.score
    }

    pub fn copy(&self) -> EscalationPreference {
        EscalationPreference {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            segment: self.segment.clone(),
            visible: self.visible,
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a escalation preference in status planned.
pub fn new_escalation_preference(code: &str, title: &str) -> EscalationPreference {
    EscalationPreference {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        segment: "".to_string(),
        visible: false,
        score: 0.0,
        status: EscalationPreferenceStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_escalation_preference("code-187", "title-3");
        assert_eq!(item.status, EscalationPreferenceStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_escalation_preference("code-187", "title-3");
        if item.move_to(EscalationPreferenceStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, EscalationPreferenceStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_escalation_preference("code-187", "title-3");
        item.status = EscalationPreferenceStatus::Cancelled;
        assert!(
            item.move_to(EscalationPreferenceStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
