use super::*;

use acme_kit::text;

/// Where a lenient timeline rule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LenientTimelineRuleStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl LenientTimelineRuleStatus {
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

impl std::fmt::Display for LenientTimelineRuleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a lenient timeline rule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct LenientTimelineRule {
    pub code: String,
    pub title: String,
    pub currency: String,
    pub amount: f64,
    pub minimum: f64,
    pub archived: bool,
    pub status: LenientTimelineRuleStatus,
}

impl LenientTimelineRule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LenientTimelineRuleStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LenientTimelineRuleStatus) -> bool {
        match self.status {
            LenientTimelineRuleStatus::Planned => {
                match next {
                    LenientTimelineRuleStatus::Scheduled | LenientTimelineRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTimelineRuleStatus::Scheduled => {
                match next {
                    LenientTimelineRuleStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTimelineRuleStatus::InProgress => {
                match next {
                    LenientTimelineRuleStatus::Completed | LenientTimelineRuleStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LenientTimelineRuleStatus::Completed => {
                match next {
                    LenientTimelineRuleStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: LenientTimelineRuleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "lenient timeline rule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn effective_volume(&self) -> f64 {
        ((self.amount + self.minimum) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LenientTimelineRule {
        LenientTimelineRule {
            code: self.code.clone(),
            title: self.title.clone(),
            currency: self.currency.clone(),
            amount: self.amount,
            minimum: self.minimum,
            archived: self.archived,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a lenient timeline rule in status planned.
pub fn new_lenient_timeline_rule(code: &str, title: &str) -> LenientTimelineRule {
    LenientTimelineRule {
        code: code.to_string(),
        title: title.to_string(),
        currency: "".to_string(),
        amount: 0.0,
        minimum: 0.0,
        archived: false,
        status: LenientTimelineRuleStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_lenient_timeline_rule("code-359", "title-911");
        assert_eq!(item.status, LenientTimelineRuleStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_scheduled() {
        let mut item = new_lenient_timeline_rule("code-359", "title-911");
        if item.move_to(LenientTimelineRuleStatus::Scheduled).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LenientTimelineRuleStatus::Scheduled, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_lenient_timeline_rule("code-359", "title-911");
        item.status = LenientTimelineRuleStatus::Cancelled;
        assert!(
            item.move_to(LenientTimelineRuleStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
