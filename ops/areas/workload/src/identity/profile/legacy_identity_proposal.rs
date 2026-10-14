use super::*;

use acme_kit::text;

/// Where a legacy identity proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum LegacyIdentityProposalStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl LegacyIdentityProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
}

impl std::fmt::Display for LegacyIdentityProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A legacy identity proposal as the identity team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct LegacyIdentityProposal {
    pub code: String,
    pub name: String,
    pub retries: i64,
    pub margin: f64,
    pub ttl_seconds: i64,
    pub attempts: i64,
    pub status: LegacyIdentityProposalStatus,
}

impl LegacyIdentityProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            LegacyIdentityProposalStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: LegacyIdentityProposalStatus) -> bool {
        match self.status {
            LegacyIdentityProposalStatus::New => {
                return next == LegacyIdentityProposalStatus::Queued || next == LegacyIdentityProposalStatus::Failed;
            }
            LegacyIdentityProposalStatus::Queued => {
                return next == LegacyIdentityProposalStatus::Done;
            }
            LegacyIdentityProposalStatus::Running => {
                match next {
                    LegacyIdentityProposalStatus::Done => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            LegacyIdentityProposalStatus::Done => {
                match next {
                    LegacyIdentityProposalStatus::Failed => {
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

    pub fn move_to(&mut self, next: LegacyIdentityProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "legacy identity proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "legacy identity proposal", self.code)
    }

    pub fn adjusted_value(&self) -> f64 {
        ((self.ttl_seconds as f64 - (self.retries as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> LegacyIdentityProposal {
        LegacyIdentityProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            retries: self.retries,
            margin: self.margin,
            ttl_seconds: self.ttl_seconds,
            attempts: self.attempts,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a legacy identity proposal in status new.
pub fn new_legacy_identity_proposal(code: &str, name: &str) -> LegacyIdentityProposal {
    LegacyIdentityProposal {
        code: code.to_string(),
        name: name.to_string(),
        retries: 0,
        margin: 0.0,
        ttl_seconds: 0,
        attempts: 0,
        status: LegacyIdentityProposalStatus::New,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_new() {
        let item = new_legacy_identity_proposal("code-678", "name-542");
        assert_eq!(item.status, LegacyIdentityProposalStatus::New, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_queued() {
        let mut item = new_legacy_identity_proposal("code-678", "name-542");
        if item.move_to(LegacyIdentityProposalStatus::Queued).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, LegacyIdentityProposalStatus::Queued, "status");
    }

    #[test]
    fn rejects_moves_from_failed() {
        let mut item = new_legacy_identity_proposal("code-678", "name-542");
        item.status = LegacyIdentityProposalStatus::Failed;
        assert!(
            item.move_to(LegacyIdentityProposalStatus::New).is_err(),
            "expected move to fail"
        );
    }
}
