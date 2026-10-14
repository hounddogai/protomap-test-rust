use super::*;

use acme_kit::text;

/// Where a batched postmortem proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum BatchedPostmortemProposalStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl BatchedPostmortemProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for BatchedPostmortemProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A batched postmortem proposal as the postmortem team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct BatchedPostmortemProposal {
    pub code: String,
    pub label: String,
    pub created_at: i64,
    pub revision: i64,
    pub minimum: f64,
    pub starts_at: i64,
    pub weight: f64,
    pub ends_at: i64,
    pub title: String,
    pub phone: String,
    pub status: BatchedPostmortemProposalStatus,
}

impl BatchedPostmortemProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            BatchedPostmortemProposalStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: BatchedPostmortemProposalStatus) -> bool {
        match self.status {
            BatchedPostmortemProposalStatus::Proposed => {
                return next == BatchedPostmortemProposalStatus::Rejected;
            }
            BatchedPostmortemProposalStatus::Approved => {
                return next == BatchedPostmortemProposalStatus::Rejected
                    || next == BatchedPostmortemProposalStatus::Withdrawn;
            }
            BatchedPostmortemProposalStatus::Rejected => {
                match next {
                    BatchedPostmortemProposalStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: BatchedPostmortemProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "batched postmortem proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        self.revision += 1;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn effective_exposure(&self) -> f64 {
        ((self.weight - (self.created_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> BatchedPostmortemProposal {
        BatchedPostmortemProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            created_at: self.created_at,
            revision: self.revision,
            minimum: self.minimum,
            starts_at: self.starts_at,
            weight: self.weight,
            ends_at: self.ends_at,
            title: self.title.clone(),
            phone: self.phone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a batched postmortem proposal in status proposed.
pub fn new_batched_postmortem_proposal(code: &str, label: &str) -> BatchedPostmortemProposal {
    BatchedPostmortemProposal {
        code: code.to_string(),
        label: label.to_string(),
        created_at: 0,
        revision: 0,
        minimum: 0.0,
        starts_at: 0,
        weight: 0.0,
        ends_at: 0,
        title: "".to_string(),
        phone: "".to_string(),
        status: BatchedPostmortemProposalStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_batched_postmortem_proposal("code-176", "label-559");
        assert_eq!(item.status, BatchedPostmortemProposalStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_batched_postmortem_proposal("code-176", "label-559");
        if item.move_to(BatchedPostmortemProposalStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, BatchedPostmortemProposalStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_batched_postmortem_proposal("code-176", "label-559");
        item.status = BatchedPostmortemProposalStatus::Withdrawn;
        assert!(
            item.move_to(BatchedPostmortemProposalStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
