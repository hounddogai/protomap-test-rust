use super::*;

use acme_kit::text;

/// Where a primary rollout proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryRolloutProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl PrimaryRolloutProposalStatus {
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

impl std::fmt::Display for PrimaryRolloutProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One primary rollout proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryRolloutProposal {
    pub code: String,
    pub label: String,
    pub ttl_seconds: i64,
    pub currency: String,
    pub updated_at: i64,
    pub ratio: f64,
    pub status: PrimaryRolloutProposalStatus,
}

impl PrimaryRolloutProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryRolloutProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryRolloutProposalStatus) -> bool {
        match self.status {
            PrimaryRolloutProposalStatus::Planned => {
                match next {
                    PrimaryRolloutProposalStatus::InProgress | PrimaryRolloutProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRolloutProposalStatus::Scheduled => {
                match next {
                    PrimaryRolloutProposalStatus::InProgress | PrimaryRolloutProposalStatus::Cancelled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryRolloutProposalStatus::InProgress => {
                return next == PrimaryRolloutProposalStatus::Cancelled;
            }
            PrimaryRolloutProposalStatus::Completed => {
                match next {
                    PrimaryRolloutProposalStatus::Cancelled => {
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

    pub fn move_to(&mut self, next: PrimaryRolloutProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary rollout proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} / {}", self.code, self.label)
    }

    pub fn net_value(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.ratio) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryRolloutProposal {
        PrimaryRolloutProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            ttl_seconds: self.ttl_seconds,
            currency: self.currency.clone(),
            updated_at: self.updated_at,
            ratio: self.ratio,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a primary rollout proposal in status planned.
pub fn new_primary_rollout_proposal(code: &str, label: &str) -> PrimaryRolloutProposal {
    PrimaryRolloutProposal {
        code: code.to_string(),
        label: label.to_string(),
        ttl_seconds: 0,
        currency: "".to_string(),
        updated_at: 0,
        ratio: 0.0,
        status: PrimaryRolloutProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_primary_rollout_proposal("code-194", "label-915");
        assert_eq!(item.status, PrimaryRolloutProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_in_progress() {
        let mut item = new_primary_rollout_proposal("code-194", "label-915");
        if item.move_to(PrimaryRolloutProposalStatus::InProgress).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryRolloutProposalStatus::InProgress, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_primary_rollout_proposal("code-194", "label-915");
        item.status = PrimaryRolloutProposalStatus::Cancelled;
        assert!(
            item.move_to(PrimaryRolloutProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
