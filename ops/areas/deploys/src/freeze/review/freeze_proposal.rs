use super::*;

use acme_kit::text;

/// Where a freeze proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FreezeProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl FreezeProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for FreezeProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A freeze proposal as the freeze team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct FreezeProposal {
    pub code: String,
    pub label: String,
    pub summary: String,
    pub cost: f64,
    pub duration_minutes: i64,
    pub source: String,
    pub ratio: f64,
    pub retries: i64,
    pub category: String,
    pub ceiling: f64,
    pub status: FreezeProposalStatus,
}

impl FreezeProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FreezeProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FreezeProposalStatus) -> bool {
        match self.status {
            FreezeProposalStatus::Proposed => {
                match next {
                    FreezeProposalStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FreezeProposalStatus::Approved => {
                return next == FreezeProposalStatus::Rejected || next == FreezeProposalStatus::Withdrawn;
            }
            FreezeProposalStatus::Rejected => {
                return next == FreezeProposalStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FreezeProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "freeze proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_load(&self) -> f64 {
        ((self.ratio - (self.duration_minutes as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FreezeProposal {
        FreezeProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            summary: self.summary.clone(),
            cost: self.cost,
            duration_minutes: self.duration_minutes,
            source: self.source.clone(),
            ratio: self.ratio,
            retries: self.retries,
            category: self.category.clone(),
            ceiling: self.ceiling,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a freeze proposal in status proposed.
pub fn new_freeze_proposal(code: &str, label: &str) -> FreezeProposal {
    FreezeProposal {
        code: code.to_string(),
        label: label.to_string(),
        summary: "".to_string(),
        cost: 0.0,
        duration_minutes: 0,
        source: "".to_string(),
        ratio: 0.0,
        retries: 0,
        category: "".to_string(),
        ceiling: 0.0,
        status: FreezeProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_freeze_proposal("code-81", "label-297");
        assert_eq!(item.status, FreezeProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_freeze_proposal("code-81", "label-297");
        if item.move_to(FreezeProposalStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, FreezeProposalStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_freeze_proposal("code-81", "label-297");
        item.status = FreezeProposalStatus::Withdrawn;
        assert!(
            item.move_to(FreezeProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
