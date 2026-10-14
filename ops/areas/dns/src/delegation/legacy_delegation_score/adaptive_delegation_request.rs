use super::*;

use acme_kit::text;

/// Where a adaptive delegation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AdaptiveDelegationRequestStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl AdaptiveDelegationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for AdaptiveDelegationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A adaptive delegation request as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveDelegationRequest {
    pub code: String,
    pub name: String,
    pub ceiling: f64,
    pub margin: f64,
    pub updated_at: i64,
    pub ttl_seconds: i64,
    pub archived: bool,
    pub discount: f64,
    pub grace_days: i64,
    pub cost: f64,
    pub status: AdaptiveDelegationRequestStatus,
}

impl AdaptiveDelegationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            AdaptiveDelegationRequestStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: AdaptiveDelegationRequestStatus) -> bool {
        match self.status {
            AdaptiveDelegationRequestStatus::Proposed => {
                match next {
                    AdaptiveDelegationRequestStatus::Approved | AdaptiveDelegationRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDelegationRequestStatus::Approved => {
                match next {
                    AdaptiveDelegationRequestStatus::Rejected | AdaptiveDelegationRequestStatus::Withdrawn => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            AdaptiveDelegationRequestStatus::Rejected => {
                return next == AdaptiveDelegationRequestStatus::Withdrawn;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: AdaptiveDelegationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "adaptive delegation request {} cannot move from {} to {}",
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

    pub fn label(&self) -> String {
        format!("{} {}", "adaptive delegation request", self.code)
    }

    pub fn projected_total(&self) -> f64 {
        ((self.ceiling - (self.ttl_seconds as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> AdaptiveDelegationRequest {
        AdaptiveDelegationRequest {
            code: self.code.clone(),
            name: self.name.clone(),
            ceiling: self.ceiling,
            margin: self.margin,
            updated_at: self.updated_at,
            ttl_seconds: self.ttl_seconds,
            archived: self.archived,
            discount: self.discount,
            grace_days: self.grace_days,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a adaptive delegation request in status proposed.
pub fn new_adaptive_delegation_request(code: &str, name: &str) -> AdaptiveDelegationRequest {
    AdaptiveDelegationRequest {
        code: code.to_string(),
        name: name.to_string(),
        ceiling: 0.0,
        margin: 0.0,
        updated_at: 0,
        ttl_seconds: 0,
        archived: false,
        discount: 0.0,
        grace_days: 0,
        cost: 0.0,
        status: AdaptiveDelegationRequestStatus::Proposed,
    }
}
