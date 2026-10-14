use super::*;

use acme_kit::text;

/// Where a release proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReleaseProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReleaseProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReleaseProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A release proposal. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct ReleaseProposal {
    pub code: String,
    pub name: String,
    pub attempts: i64,
    pub phone: String,
    pub checksum: String,
    pub status: ReleaseProposalStatus,
}

impl ReleaseProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReleaseProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReleaseProposalStatus) -> bool {
        match self.status {
            ReleaseProposalStatus::Requested => {
                match next {
                    ReleaseProposalStatus::Revoked | ReleaseProposalStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReleaseProposalStatus::Granted => {
                return next == ReleaseProposalStatus::Expired;
            }
            ReleaseProposalStatus::Revoked => {
                match next {
                    ReleaseProposalStatus::Expired => {
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

    pub fn move_to(&mut self, next: ReleaseProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "release proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "release proposal", self.code)
    }

    pub fn measured_value(&self) -> f64 {
        self.attempts as f64
    }

    pub fn copy(&self) -> ReleaseProposal {
        ReleaseProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            attempts: self.attempts,
            phone: self.phone.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a release proposal in status requested.
pub fn new_release_proposal(code: &str, name: &str) -> ReleaseProposal {
    ReleaseProposal {
        code: code.to_string(),
        name: name.to_string(),
        attempts: 0,
        phone: "".to_string(),
        checksum: "".to_string(),
        status: ReleaseProposalStatus::Requested,
    }
}
