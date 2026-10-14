use super::*;

use acme_kit::text;

/// Where a fallback plan target is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FallbackPlanTargetStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FallbackPlanTargetStatus {
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

impl std::fmt::Display for FallbackPlanTargetStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One fallback plan target, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct FallbackPlanTarget {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub contact_email: String,
    pub namespace: String,
    pub burst: i64,
    pub quantity: i64,
    pub grace_days: i64,
    pub description: String,
    pub status: FallbackPlanTargetStatus,
}

impl FallbackPlanTarget {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FallbackPlanTargetStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FallbackPlanTargetStatus) -> bool {
        match self.status {
            FallbackPlanTargetStatus::Open => {
                match next {
                    FallbackPlanTargetStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPlanTargetStatus::InReview => {
                match next {
                    FallbackPlanTargetStatus::Resolved | FallbackPlanTargetStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPlanTargetStatus::Resolved => {
                match next {
                    FallbackPlanTargetStatus::Reopened | FallbackPlanTargetStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FallbackPlanTargetStatus::Reopened => {
                match next {
                    FallbackPlanTargetStatus::Archived => {
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

    pub fn move_to(&mut self, next: FallbackPlanTargetStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "fallback plan target {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.name)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.burst as f64 + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FallbackPlanTarget {
        FallbackPlanTarget {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            contact_email: self.contact_email.clone(),
            namespace: self.namespace.clone(),
            burst: self.burst,
            quantity: self.quantity,
            grace_days: self.grace_days,
            description: self.description.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.namespace)
    }
}

/// Starts a fallback plan target in status open.
pub fn new_fallback_plan_target(code: &str, name: &str) -> FallbackPlanTarget {
    FallbackPlanTarget {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        contact_email: "".to_string(),
        namespace: "".to_string(),
        burst: 0,
        quantity: 0,
        grace_days: 0,
        description: "".to_string(),
        status: FallbackPlanTargetStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_fallback_plan_target("code-851", "name-559");
        assert_eq!(item.status, FallbackPlanTargetStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_review() {
        let mut item = new_fallback_plan_target("code-851", "name-559");
        if item.move_to(FallbackPlanTargetStatus::InReview).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FallbackPlanTargetStatus::InReview, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_fallback_plan_target("code-851", "name-559");
        item.status = FallbackPlanTargetStatus::Archived;
        assert!(
            item.move_to(FallbackPlanTargetStatus::Open).is_err(),
            "expected move to fail"
        );
    }
}
