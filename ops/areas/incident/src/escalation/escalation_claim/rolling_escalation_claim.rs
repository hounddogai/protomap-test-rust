use super::*;

use acme_kit::text;

/// Where a rolling escalation claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RollingEscalationClaimStatus {
    #[default]
    Draft,
    Pending,
    Active,
    Suspended,
    Closed,
}

impl RollingEscalationClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Draft => "draft",
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Suspended => "suspended",
            Self::Closed => "closed",
        }
    }
}

impl std::fmt::Display for RollingEscalationClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A rolling escalation claim as the escalation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct RollingEscalationClaim {
    pub code: String,
    pub name: String,
    pub tenant_id: String,
    pub ttl_seconds: i64,
    pub issuer: String,
    pub active: bool,
    pub approved: bool,
    pub ceiling: f64,
    pub category: String,
    pub currency: String,
    pub status: RollingEscalationClaimStatus,
}

impl RollingEscalationClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RollingEscalationClaimStatus::Closed => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RollingEscalationClaimStatus) -> bool {
        match self.status {
            RollingEscalationClaimStatus::Draft => {
                return next == RollingEscalationClaimStatus::Pending || next == RollingEscalationClaimStatus::Active;
            }
            RollingEscalationClaimStatus::Pending => {
                return next == RollingEscalationClaimStatus::Active || next == RollingEscalationClaimStatus::Closed;
            }
            RollingEscalationClaimStatus::Active => {
                return next == RollingEscalationClaimStatus::Suspended || next == RollingEscalationClaimStatus::Closed;
            }
            RollingEscalationClaimStatus::Suspended => {
                return next == RollingEscalationClaimStatus::Closed;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RollingEscalationClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "rolling escalation claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn weighted_total(&self) -> f64 {
        ((self.ttl_seconds as f64 + self.ceiling) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RollingEscalationClaim {
        RollingEscalationClaim {
            code: self.code.clone(),
            name: self.name.clone(),
            tenant_id: self.tenant_id.clone(),
            ttl_seconds: self.ttl_seconds,
            issuer: self.issuer.clone(),
            active: self.active,
            approved: self.approved,
            ceiling: self.ceiling,
            category: self.category.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.category)
    }
}

/// Starts a rolling escalation claim in status draft.
pub fn new_rolling_escalation_claim(code: &str, name: &str) -> RollingEscalationClaim {
    RollingEscalationClaim {
        code: code.to_string(),
        name: name.to_string(),
        tenant_id: "".to_string(),
        ttl_seconds: 0,
        issuer: "".to_string(),
        active: false,
        approved: false,
        ceiling: 0.0,
        category: "".to_string(),
        currency: "".to_string(),
        status: RollingEscalationClaimStatus::Draft,
    }
}
