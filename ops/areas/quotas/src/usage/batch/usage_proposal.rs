use super::*;

use acme_kit::text;

/// Where a usage proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum UsageProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl UsageProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for UsageProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A usage proposal as the usage team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct UsageProposal {
    pub code: String,
    pub title: String,
    pub port: i64,
    pub approved: bool,
    pub principal: String,
    pub ceiling: f64,
    pub cipher: String,
    pub grace_days: i64,
    pub scope: String,
    pub checksum: String,
    pub status: UsageProposalStatus,
}

impl UsageProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            UsageProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: UsageProposalStatus) -> bool {
        match self.status {
            UsageProposalStatus::Requested => {
                return next == UsageProposalStatus::Granted || next == UsageProposalStatus::Revoked;
            }
            UsageProposalStatus::Granted => {
                match next {
                    UsageProposalStatus::Revoked | UsageProposalStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            UsageProposalStatus::Revoked => {
                return next == UsageProposalStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: UsageProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "usage proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.title)
    }

    pub fn net_total(&self) -> f64 {
        ((self.grace_days as f64 - (self.port as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> UsageProposal {
        UsageProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            port: self.port,
            approved: self.approved,
            principal: self.principal.clone(),
            ceiling: self.ceiling,
            cipher: self.cipher.clone(),
            grace_days: self.grace_days,
            scope: self.scope.clone(),
            checksum: self.checksum.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a usage proposal in status requested.
pub fn new_usage_proposal(code: &str, title: &str) -> UsageProposal {
    UsageProposal {
        code: code.to_string(),
        title: title.to_string(),
        port: 0,
        approved: false,
        principal: "".to_string(),
        ceiling: 0.0,
        cipher: "".to_string(),
        grace_days: 0,
        scope: "".to_string(),
        checksum: "".to_string(),
        status: UsageProposalStatus::Requested,
    }
}
