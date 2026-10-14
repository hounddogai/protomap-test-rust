use super::*;

use acme_kit::text;

/// Where a plan profile is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanProfileStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl PlanProfileStatus {
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

impl std::fmt::Display for PlanProfileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan profile as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanProfile {
    pub code: String,
    pub label: String,
    pub score: f64,
    pub description: String,
    pub tenant_id: String,
    pub contact_email: String,
    pub revision: i64,
    pub status: PlanProfileStatus,
}

impl PlanProfile {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanProfileStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanProfileStatus) -> bool {
        match self.status {
            PlanProfileStatus::Open => {
                return next == PlanProfileStatus::Reopened || next == PlanProfileStatus::Archived;
            }
            PlanProfileStatus::InReview => {
                return next == PlanProfileStatus::Resolved || next == PlanProfileStatus::Reopened;
            }
            PlanProfileStatus::Resolved => {
                return next == PlanProfileStatus::Reopened || next == PlanProfileStatus::Archived;
            }
            PlanProfileStatus::Reopened => {
                match next {
                    PlanProfileStatus::Archived => {
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

    pub fn move_to(&mut self, next: PlanProfileStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan profile {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.score - (self.revision as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanProfile {
        PlanProfile {
            code: self.code.clone(),
            label: self.label.clone(),
            score: self.score,
            description: self.description.clone(),
            tenant_id: self.tenant_id.clone(),
            contact_email: self.contact_email.clone(),
            revision: self.revision,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a plan profile in status open.
pub fn new_plan_profile(code: &str, label: &str) -> PlanProfile {
    PlanProfile {
        code: code.to_string(),
        label: label.to_string(),
        score: 0.0,
        description: "".to_string(),
        tenant_id: "".to_string(),
        contact_email: "".to_string(),
        revision: 0,
        status: PlanProfileStatus::Open,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_open() {
        let item = new_plan_profile("code-194", "label-337");
        assert_eq!(item.status, PlanProfileStatus::Open, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_reopened() {
        let mut item = new_plan_profile("code-194", "label-337");
        if item.move_to(PlanProfileStatus::Reopened).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanProfileStatus::Reopened, "status");
    }

    #[test]
    fn rejects_moves_from_archived() {
        let mut item = new_plan_profile("code-194", "label-337");
        item.status = PlanProfileStatus::Archived;
        assert!(item.move_to(PlanProfileStatus::Open).is_err(), "expected move to fail");
    }
}
