use super::*;

use acme_kit::text;

/// Where a delegation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationClaimStatus {
    #[default]
    New,
    Queued,
    Running,
    Done,
    Failed,
}

impl DelegationClaimStatus {
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

impl std::fmt::Display for DelegationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a delegation claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationClaim {
    pub code: String,
    pub name: String,
    pub approved: bool,
    pub ceiling: f64,
    pub burst: i64,
    pub tenant_id: String,
    pub cluster: String,
    pub key_id: String,
    pub capacity: i64,
    pub status: DelegationClaimStatus,
}

impl DelegationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationClaimStatus::Failed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationClaimStatus) -> bool {
        match self.status {
            DelegationClaimStatus::New => {
                return next == DelegationClaimStatus::Done || next == DelegationClaimStatus::Failed;
            }
            DelegationClaimStatus::Queued => {
                match next {
                    DelegationClaimStatus::Running | DelegationClaimStatus::Failed => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationClaimStatus::Running => {
                return next == DelegationClaimStatus::Done || next == DelegationClaimStatus::Failed;
            }
            DelegationClaimStatus::Done => {
                match next {
                    DelegationClaimStatus::Failed => {
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

    pub fn move_to(&mut self, next: DelegationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "delegation claim", self.code)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.burst as f64 + (self.capacity as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationClaim {
        DelegationClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            approved: self.approved,
            ceiling: self.ceiling,
            burst: self.burst,
            tenant_id: self.tenant_id.clone(),
            cluster: self.cluster.clone(),
            key_id: self.key_id.clone(),
            capacity: self.capacity,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.cluster)
    }
}

/// Starts a delegation claim in status new.
pub fn new_delegation_claim(code: &str, name: &str) -> DelegationClaim {
    DelegationClaim {
        code: code.to_string(),
        name: name.to_string(),
        approved: false,
        ceiling: 0.0,
        burst: 0,
        tenant_id: "".to_string(),
        cluster: "".to_string(),
        key_id: "".to_string(),
        capacity: 0,
        status: DelegationClaimStatus::New,
    }
}
