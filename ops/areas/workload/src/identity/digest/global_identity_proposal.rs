use super::*;

use acme_kit::text;

/// Where a global identity proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum GlobalIdentityProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl GlobalIdentityProposalStatus {
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

impl std::fmt::Display for GlobalIdentityProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One global identity proposal, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct GlobalIdentityProposal {
    pub code: String,
    pub title: String,
    pub created_at: i64,
    pub ceiling: f64,
    pub region: String,
    pub scope: String,
    pub category: String,
    pub status: GlobalIdentityProposalStatus,
}

impl GlobalIdentityProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            GlobalIdentityProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: GlobalIdentityProposalStatus) -> bool {
        match self.status {
            GlobalIdentityProposalStatus::Planned => {
                return next == GlobalIdentityProposalStatus::Completed
                    || next == GlobalIdentityProposalStatus::Cancelled;
            }
            GlobalIdentityProposalStatus::Scheduled => {
                match next {
                    GlobalIdentityProposalStatus::InProgress => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityProposalStatus::InProgress => {
                match next {
                    GlobalIdentityProposalStatus::Completed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            GlobalIdentityProposalStatus::Completed => {
                return next == GlobalIdentityProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: GlobalIdentityProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "global identity proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.created_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.title)
    }

    pub fn weighted_exposure(&self) -> f64 {
        ((self.created_at as f64) * self.ceiling * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> GlobalIdentityProposal {
        GlobalIdentityProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            created_at: self.created_at,
            ceiling: self.ceiling,
            region: self.region.clone(),
            scope: self.scope.clone(),
            category: self.category.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.region)
    }
}

/// Starts a global identity proposal in status planned.
pub fn new_global_identity_proposal(code: &str, title: &str) -> GlobalIdentityProposal {
    GlobalIdentityProposal {
        code: code.to_string(),
        title: title.to_string(),
        created_at: 0,
        ceiling: 0.0,
        region: "".to_string(),
        scope: "".to_string(),
        category: "".to_string(),
        status: GlobalIdentityProposalStatus::Planned,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_planned() {
        let item = new_global_identity_proposal("code-682", "title-144");
        assert_eq!(item.status, GlobalIdentityProposalStatus::Planned, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_completed() {
        let mut item = new_global_identity_proposal("code-682", "title-144");
        if item.move_to(GlobalIdentityProposalStatus::Completed).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, GlobalIdentityProposalStatus::Completed, "status");
    }

    #[test]
    fn rejects_moves_from_cancelled() {
        let mut item = new_global_identity_proposal("code-682", "title-144");
        item.status = GlobalIdentityProposalStatus::Cancelled;
        assert!(
            item.move_to(GlobalIdentityProposalStatus::Planned).is_err(),
            "expected move to fail"
        );
    }
}
