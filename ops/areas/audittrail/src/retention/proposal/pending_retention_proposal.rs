use super::*;

use acme_kit::text;

/// Where a pending retention proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PendingRetentionProposalStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl PendingRetentionProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for PendingRetentionProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a pending retention proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingRetentionProposal {
    pub code: String,
    pub name: String,
    pub contact_email: String,
    pub minimum: f64,
    pub rate: f64,
    pub starts_at: i64,
    pub sequence: i64,
    pub status: PendingRetentionProposalStatus,
}

impl PendingRetentionProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PendingRetentionProposalStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PendingRetentionProposalStatus) -> bool {
        match self.status {
            PendingRetentionProposalStatus::Created => {
                return next == PendingRetentionProposalStatus::Validated;
            }
            PendingRetentionProposalStatus::Validated => {
                return next == PendingRetentionProposalStatus::Published;
            }
            PendingRetentionProposalStatus::Published => {
                return next == PendingRetentionProposalStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PendingRetentionProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "pending retention proposal {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.starts_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|elapsed| elapsed.as_secs() as i64)
            .unwrap_or_default();
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "pending retention proposal", self.code)
    }

    pub fn net_value(&self) -> f64 {
        ((self.starts_at as f64 - (self.sequence as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PendingRetentionProposal {
        PendingRetentionProposal {
            code: self.code.clone(),
            name: self.name.clone(),
            contact_email: self.contact_email.clone(),
            minimum: self.minimum,
            rate: self.rate,
            starts_at: self.starts_at,
            sequence: self.sequence,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a pending retention proposal in status created.
pub fn new_pending_retention_proposal(code: &str, name: &str) -> PendingRetentionProposal {
    PendingRetentionProposal {
        code: code.to_string(),
        name: name.to_string(),
        contact_email: "".to_string(),
        minimum: 0.0,
        rate: 0.0,
        starts_at: 0,
        sequence: 0,
        status: PendingRetentionProposalStatus::Created,
    }
}
