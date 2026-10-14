use super::*;

use acme_kit::text;

/// Where a primary plan proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryPlanProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryPlanProposalStatus {
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

impl std::fmt::Display for PrimaryPlanProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary plan proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryPlanProposal {
    pub code: String,
    pub title: String,
    pub visible: bool,
    pub quota_units: i64,
    pub currency: String,
    pub expires_at: i64,
    pub capacity: i64,
    pub status: PrimaryPlanProposalStatus,
}

impl PrimaryPlanProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryPlanProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryPlanProposalStatus) -> bool {
        match self.status {
            PrimaryPlanProposalStatus::Planned => {
                match next {
                    PrimaryPlanProposalStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPlanProposalStatus::Scheduled => {
                return next == PrimaryPlanProposalStatus::Cancelled;
            }
            PrimaryPlanProposalStatus::InProgress => {
                match next {
                    PrimaryPlanProposalStatus::Completed | PrimaryPlanProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryPlanProposalStatus::Completed => {
                match next {
                    PrimaryPlanProposalStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryPlanProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary plan proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.expires_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.quota_units as f64) * (self.capacity as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryPlanProposal {
        PrimaryPlanProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            visible: self.visible,
            quota_units: self.quota_units,
            currency: self.currency.clone(),
            expires_at: self.expires_at,
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a primary plan proposal in status planned.
pub fn new_primary_plan_proposal(code: &str, title: &str) -> PrimaryPlanProposal {
    PrimaryPlanProposal {
        code: code.to_string(),
        title: title.to_string(),
        visible: false,
        quota_units: 0,
        currency: "".to_string(),
        expires_at: 0,
        capacity: 0,
        status: PrimaryPlanProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_plan_proposal("code-486", "title-116");
        assert_eq!(item.status, PrimaryPlanProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_primary_plan_proposal("code-486", "title-116");
        if item.move_to(PrimaryPlanProposalStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryPlanProposalStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_plan_proposal("code-486", "title-116");
        item.status = PrimaryPlanProposalStatus::Cancelled;
        assert!(
            item.move_to(PrimaryPlanProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
