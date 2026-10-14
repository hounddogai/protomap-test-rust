use super::*;

use acme_kit::text;

/// Where a federation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum FederationClaimStatus {
    #[default]
    Open,
    InReview,
    Resolved,
    Reopened,
    Archived,
}

impl FederationClaimStatus {
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

impl std::fmt::Display for FederationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A federation claim. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct FederationClaim {
    pub code: String,
    pub label: String,
    pub source: String,
    pub limit: i64,
    pub expires_at: i64,
    pub status: FederationClaimStatus,
}

impl FederationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            FederationClaimStatus::Archived => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: FederationClaimStatus) -> bool {
        match self.status {
            FederationClaimStatus::Open => {
                match next {
                    FederationClaimStatus::Reopened | FederationClaimStatus::Archived => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationClaimStatus::InReview => {
                match next {
                    FederationClaimStatus::Resolved => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationClaimStatus::Resolved => {
                match next {
                    FederationClaimStatus::Reopened => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            FederationClaimStatus::Reopened => {
                return next == FederationClaimStatus::Archived;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: FederationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "federation claim {} cannot move from {} to {}",
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

    pub fn display_label(&self) -> String {
        format!("{} ({})", self.code, self.label)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.expires_at as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> FederationClaim {
        FederationClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            source: self.source.clone(),
            limit: self.limit,
            expires_at: self.expires_at,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.source)
    }
}

/// Starts a federation claim in status open.
pub fn new_federation_claim(code: &str, label: &str) -> FederationClaim {
    FederationClaim {
        code: code.to_string(),
        label: label.to_string(),
        source: "".to_string(),
        limit: 0,
        expires_at: 0,
        status: FederationClaimStatus::Open,
    }
}
