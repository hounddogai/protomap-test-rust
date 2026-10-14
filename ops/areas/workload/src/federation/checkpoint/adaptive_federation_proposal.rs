use super::*;

use acme_kit::text;

/// Where a adaptive federation proposal is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveFederationProposalStatus {
    #[default]
    Planned,
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

impl AdaptiveFederationProposalStatus {
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

impl std::fmt::Display for AdaptiveFederationProposalStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a adaptive federation proposal and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveFederationProposal {
    pub code: String,
    pub label: String,
    pub margin: f64,
    pub issuer: String,
    pub batch_size: i64,
    pub minimum: f64,
    pub external_ref: String,
    pub updated_at: i64,
    pub quota_units: i64,
    pub scope: String,
    pub status: AdaptiveFederationProposalStatus,
}

impl AdaptiveFederationProposal {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveFederationProposalStatus::Cancelled => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveFederationProposalStatus) -> bool {
        match self.status {
            AdaptiveFederationProposalStatus::Planned => {
                match next {
                    AdaptiveFederationProposalStatus::Scheduled => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveFederationProposalStatus::Scheduled => {
                return next == AdaptiveFederationProposalStatus::Completed;
            }
            AdaptiveFederationProposalStatus::InProgress => {
                return next == AdaptiveFederationProposalStatus::Completed
                    || next == AdaptiveFederationProposalStatus::Cancelled;
            }
            AdaptiveFederationProposalStatus::Completed => {
                return next == AdaptiveFederationProposalStatus::Cancelled;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveFederationProposalStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive federation proposal {} cannot move from {} to {}",
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
        format!("{} {}", "adaptive federation proposal", self.code)
    }

    pub fn projected_load(&self) -> f64 {
        (self.minimum * (self.batch_size as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveFederationProposal {
        AdaptiveFederationProposal {
            code: self.code.clone(),
            label: self.label.clone(),
            margin: self.margin,
            issuer: self.issuer.clone(),
            batch_size: self.batch_size,
            minimum: self.minimum,
            external_ref: self.external_ref.clone(),
            updated_at: self.updated_at,
            quota_units: self.quota_units,
            scope: self.scope.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a adaptive federation proposal in status planned.
pub fn new_adaptive_federation_proposal(code: &str, label: &str) -> AdaptiveFederationProposal {
    AdaptiveFederationProposal {
        code: code.to_string(),
        label: label.to_string(),
        margin: 0.0,
        issuer: "".to_string(),
        batch_size: 0,
        minimum: 0.0,
        external_ref: "".to_string(),
        updated_at: 0,
        quota_units: 0,
        scope: "".to_string(),
        status: AdaptiveFederationProposalStatus::Planned,
    }
}
