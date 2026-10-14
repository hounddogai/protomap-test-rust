use super::*;

use acme_kit::text;

/// Where a redaction plan is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RedactionPlanStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl RedactionPlanStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for RedactionPlanStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A redaction plan. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct RedactionPlan {
    pub code: String,
    pub label: String,
    pub budget: f64,
    pub window_days: i64,
    pub rotation_days: i64,
    pub checksum: String,
    pub score: f64,
    pub status: RedactionPlanStatus,
}

impl RedactionPlan {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RedactionPlanStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RedactionPlanStatus) -> bool {
        match self.status {
            RedactionPlanStatus::Requested => {
                return next == RedactionPlanStatus::Granted || next == RedactionPlanStatus::Expired;
            }
            RedactionPlanStatus::Granted => {
                match next {
                    RedactionPlanStatus::Revoked | RedactionPlanStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            RedactionPlanStatus::Revoked => {
                match next {
                    RedactionPlanStatus::Expired => {
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

    pub fn move_to(&mut self, next: RedactionPlanStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "redaction plan {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{}: {}", self.code, self.label)
    }

    pub fn weighted_value(&self) -> f64 {
        ((self.score + self.budget) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RedactionPlan {
        RedactionPlan {
            code: self.code.clone(),
            label: self.label.clone(),
            budget: self.budget,
            window_days: self.window_days,
            rotation_days: self.rotation_days,
            checksum: self.checksum.clone(),
            score: self.score,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a redaction plan in status requested.
pub fn new_redaction_plan(code: &str, label: &str) -> RedactionPlan {
    RedactionPlan {
        code: code.to_string(),
        label: label.to_string(),
        budget: 0.0,
        window_days: 0,
        rotation_days: 0,
        checksum: "".to_string(),
        score: 0.0,
        status: RedactionPlanStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_redaction_plan("code-417", "label-614");
        assert_eq!(item.status, RedactionPlanStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_redaction_plan("code-417", "label-614");
        if item.move_to(RedactionPlanStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RedactionPlanStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_redaction_plan("code-417", "label-614");
        item.status = RedactionPlanStatus::Expired;
        assert!(
            item.move_to(RedactionPlanStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
