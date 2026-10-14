use super::*;

use acme_kit::text;

/// Where a federation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationProposalStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl FederationProposalStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for FederationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a federation proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationProposal {
    pub code: String,
    pub title: String,
    pub timezone: String,
    pub fingerprint: String,
    pub ceiling: f64,
    pub expires_at: i64,
    pub rotation_days: i64,
    pub amount: f64,
    pub status: FederationProposalStatus,
}

impl FederationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationProposalStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationProposalStatus) -> bool {
        match self.status {
            FederationProposalStatus::Requested => {
                match next {
                    FederationProposalStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationProposalStatus::Granted => {
                return next == FederationProposalStatus::Expired;
            }
            FederationProposalStatus::Revoked => {
                return next == FederationProposalStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation proposal {} cannot move from {} to {}",
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
        format!("{} ({})", self.code, self.title)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.expires_at as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationProposal {
        FederationProposal {
            code: self.code.clone(),
            title: self.title.clone(),
            timezone: self.timezone.clone(),
            fingerprint: self.fingerprint.clone(),
            ceiling: self.ceiling,
            expires_at: self.expires_at,
            rotation_days: self.rotation_days,
            amount: self.amount,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.title)
    }
}

/// Starts a federation proposal in status requested.
pub fn new_federation_proposal(code: &str, title: &str) -> FederationProposal {
    FederationProposal {
        code: code.to_string(),
        title: title.to_string(),
        timezone: "".to_string(),
        fingerprint: "".to_string(),
        ceiling: 0.0,
        expires_at: 0,
        rotation_days: 0,
        amount: 0.0,
        status: FederationProposalStatus::Requested,
    }
}
