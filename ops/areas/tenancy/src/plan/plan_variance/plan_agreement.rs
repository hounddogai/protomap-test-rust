use super::*;

use acme_kit::text;

/// Where a plan agreement is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanAgreementStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PlanAgreementStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PlanAgreementStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A plan agreement as the plan team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAgreement {
    pub code: String,
    pub title: String,
    pub segment: String,
    pub phone: String,
    pub notes: String,
    pub ttl_seconds: i64,
    pub status: PlanAgreementStatus,
}

impl PlanAgreement {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanAgreementStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanAgreementStatus) -> bool {
        match self.status {
            PlanAgreementStatus::Proposed => {
                return next == PlanAgreementStatus::Withdrawn;
            }
            PlanAgreementStatus::Approved => {
                return next == PlanAgreementStatus::Rejected;
            }
            PlanAgreementStatus::Rejected => {
                match next {
                    PlanAgreementStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PlanAgreementStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan agreement {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan agreement", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.ttl_seconds as f64
    }

    pub fn copy(&self) -> PlanAgreement {
        PlanAgreement {
            code: self.code.clone(),
            title: self.title.clone(),
            segment: self.segment.clone(),
            phone: self.phone.clone(),
            notes: self.notes.clone(),
            ttl_seconds: self.ttl_seconds,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.segment)
    }
}

/// Starts a plan agreement in status proposed.
pub fn new_plan_agreement(code: &str, title: &str) -> PlanAgreement {
    PlanAgreement {
        code: code.to_string(),
        title: title.to_string(),
        segment: "".to_string(),
        phone: "".to_string(),
        notes: "".to_string(),
        ttl_seconds: 0,
        status: PlanAgreementStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_plan_agreement("code-630", "title-231");
        assert_eq!(item.status, PlanAgreementStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_withdrawn() {
        let mut item = new_plan_agreement("code-630", "title-231");
        if item.move_to(PlanAgreementStatus::Withdrawn).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanAgreementStatus::Withdrawn, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_plan_agreement("code-630", "title-231");
        item.status = PlanAgreementStatus::Withdrawn;
        assert!(
            item.move_to(PlanAgreementStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
