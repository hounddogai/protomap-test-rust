use super::*;

use acme_kit::text;

/// Where a trust domain checkpoint is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum TrustDomainCheckpointStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl TrustDomainCheckpointStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::InReview => "in_review",
            Self::Resolved => "resolved",
            Self::Reopened => "reopened",
            Self::Archived => "archived",
        }
    }
}

impl std::fmt::Display for TrustDomainCheckpointStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A trust domain checkpoint. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct TrustDomainCheckpoint {
    pub code: String,
    pub name: String,
    pub rate: f64,
    pub quantity: i64,
    pub cluster: String,
    pub issuer: String,
    pub timezone: String,
    pub status: TrustDomainCheckpointStatus,
}

impl TrustDomainCheckpoint {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            TrustDomainCheckpointStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: TrustDomainCheckpointStatus) -> bool {
        match self.status {
            TrustDomainCheckpointStatus::Open => {
                match next {
                    TrustDomainCheckpointStatus::InReview => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            TrustDomainCheckpointStatus::InReview => {
                return next == TrustDomainCheckpointStatus::Resolved || next == TrustDomainCheckpointStatus::Reopened;
            }
            TrustDomainCheckpointStatus::Resolved => {
                return next == TrustDomainCheckpointStatus::Archived;
            }
            TrustDomainCheckpointStatus::Reopened => {
                match next {
                    TrustDomainCheckpointStatus::Archived => {
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

    pub fn move_to(&mut self, next: TrustDomainCheckpointStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "trust domain checkpoint {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "trust domain checkpoint", self.code)
    }

    pub fn adjusted_volume(&self) -> f64 {
        ((self.quantity as f64 + self.rate) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> TrustDomainCheckpoint {
        TrustDomainCheckpoint {
            code: self.code.clone(),
            name: self.name.clone(),
            rate: self.rate,
            quantity: self.quantity,
            cluster: self.cluster.clone(),
            issuer: self.issuer.clone(),
            timezone: self.timezone.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a trust domain checkpoint in status open.
pub fn new_trust_domain_checkpoint(code: &str, name: &str) -> TrustDomainCheckpoint {
    TrustDomainCheckpoint {
        code: code.to_string(),
        name: name.to_string(),
        rate: 0.0,
        quantity: 0,
        cluster: "".to_string(),
        issuer: "".to_string(),
        timezone: "".to_string(),
        status: TrustDomainCheckpointStatus::Open,
    }
}
