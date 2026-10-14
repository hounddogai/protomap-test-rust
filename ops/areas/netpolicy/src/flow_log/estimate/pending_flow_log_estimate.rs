use super::*;

use acme_kit::text;

/// Where a pending flow log estimate is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingFlowLogEstimateStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PendingFlowLogEstimateStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PendingFlowLogEstimateStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A pending flow log estimate as the flow log team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingFlowLogEstimate {
    pub code: String,
    pub title: String,
    pub summary: String,
    pub phone: String,
    pub notes: String,
    pub quantity: i64,
    pub status: PendingFlowLogEstimateStatus,
}

impl PendingFlowLogEstimate {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingFlowLogEstimateStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingFlowLogEstimateStatus) -> bool {
        match self.status {
            PendingFlowLogEstimateStatus::Proposed => {
                match next {
                    PendingFlowLogEstimateStatus::Approved | PendingFlowLogEstimateStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PendingFlowLogEstimateStatus::Approved => {
                return next == PendingFlowLogEstimateStatus::Rejected
                    || next == PendingFlowLogEstimateStatus::Withdrawn;
            }
            PendingFlowLogEstimateStatus::Rejected => {
                match next {
                    PendingFlowLogEstimateStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PendingFlowLogEstimateStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending flow log estimate {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{}: {}", self.code, self.title)
    }

    pub fn measured_value(&self) -> f64 {
        self.quantity as f64
    }

    pub fn copy(&self) -> PendingFlowLogEstimate {
        PendingFlowLogEstimate {
            code: self.code.clone(),
            title: self.title.clone(),
            summary: self.summary.clone(),
            phone: self.phone.clone(),
            notes: self.notes.clone(),
            quantity: self.quantity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a pending flow log estimate in status proposed.
pub fn new_pending_flow_log_estimate(code: &str, title: &str) -> PendingFlowLogEstimate {
    PendingFlowLogEstimate {
        code: code.to_string(),
        title: title.to_string(),
        summary: "".to_string(),
        phone: "".to_string(),
        notes: "".to_string(),
        quantity: 0,
        status: PendingFlowLogEstimateStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_pending_flow_log_estimate("code-572", "title-914");
        assert_eq!(item.status, PendingFlowLogEstimateStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_approved() {
        let mut item = new_pending_flow_log_estimate("code-572", "title-914");
        if item.move_to(PendingFlowLogEstimateStatus::Approved).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PendingFlowLogEstimateStatus::Approved, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_pending_flow_log_estimate("code-572", "title-914");
        item.status = PendingFlowLogEstimateStatus::Withdrawn;
        assert!(
            item.move_to(PendingFlowLogEstimateStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
