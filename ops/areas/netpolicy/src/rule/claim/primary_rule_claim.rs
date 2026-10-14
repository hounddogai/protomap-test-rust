use super::*;

use acme_kit::text;

/// Where a primary rule claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRuleClaimStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryRuleClaimStatus {
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

impl std::fmt::Display for PrimaryRuleClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary rule claim, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRuleClaim {
    pub code: String,
    pub title: String,
    pub category: String,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub status: PrimaryRuleClaimStatus,
}

impl PrimaryRuleClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRuleClaimStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRuleClaimStatus) -> bool {
        match self.status {
            PrimaryRuleClaimStatus::Planned => {
                return next == PrimaryRuleClaimStatus::Completed;
            }
            PrimaryRuleClaimStatus::Scheduled => {
                return next == PrimaryRuleClaimStatus::InProgress;
            }
            PrimaryRuleClaimStatus::InProgress => {
                return next == PrimaryRuleClaimStatus::Completed;
            }
            PrimaryRuleClaimStatus::Completed => {
                return next == PrimaryRuleClaimStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PrimaryRuleClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rule claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + (self.attempts as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRuleClaim {
        PrimaryRuleClaim {
            code: self.code.clone(),
            title: self.title.clone(),
            category: self.category.clone(),
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a primary rule claim in status planned.
pub fn new_primary_rule_claim(code: &str, title: &str) -> PrimaryRuleClaim {
    PrimaryRuleClaim {
        code: code.to_string(),
        title: title.to_string(),
        category: "".to_string(),
        ttl_seconds: 0,
        attempts: 0,
        status: PrimaryRuleClaimStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_rule_claim("code-536", "title-966");
        assert_eq!(item.status, PrimaryRuleClaimStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_primary_rule_claim("code-536", "title-966");
        if item.move_to(PrimaryRuleClaimStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRuleClaimStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_rule_claim("code-536", "title-966");
        item.status = PrimaryRuleClaimStatus::Cancelled;
        assert!(
            item.move_to(PrimaryRuleClaimStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
