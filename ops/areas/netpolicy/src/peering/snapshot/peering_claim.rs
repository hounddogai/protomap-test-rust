use super::*;

use acme_kit::text;

/// Where a peering claim is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PeeringClaimStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PeeringClaimStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PeeringClaimStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a peering claim and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct PeeringClaim {
    pub code: String,
    pub label: String,
    pub description: String,
    pub budget: f64,
    pub visible: bool,
    pub cipher: String,
    pub rotation_days: i64,
    pub duration_minutes: i64,
    pub timezone: String,
    pub owner_id: String,
    pub priority: i64,
    pub status: PeeringClaimStatus,
}

impl PeeringClaim {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PeeringClaimStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PeeringClaimStatus) -> bool {
        match self.status {
            PeeringClaimStatus::Requested => {
                return next == PeeringClaimStatus::Granted || next == PeeringClaimStatus::Revoked;
            }
            PeeringClaimStatus::Granted => {
                return next == PeeringClaimStatus::Revoked || next == PeeringClaimStatus::Expired;
            }
            PeeringClaimStatus::Revoked => {
                return next == PeeringClaimStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PeeringClaimStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "peering claim {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "peering claim", self.code)
    }

    pub fn effective_load(&self) -> f64 {
        ((self.priority as f64) * (self.rotation_days as f64) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PeeringClaim {
        PeeringClaim {
            code: self.code.clone(),
            label: self.label.clone(),
            description: self.description.clone(),
            budget: self.budget,
            visible: self.visible,
            cipher: self.cipher.clone(),
            rotation_days: self.rotation_days,
            duration_minutes: self.duration_minutes,
            timezone: self.timezone.clone(),
            owner_id: self.owner_id.clone(),
            priority: self.priority,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a peering claim in status requested.
pub fn new_peering_claim(code: &str, label: &str) -> PeeringClaim {
    PeeringClaim {
        code: code.to_string(),
        label: label.to_string(),
        description: "".to_string(),
        budget: 0.0,
        visible: false,
        cipher: "".to_string(),
        rotation_days: 0,
        duration_minutes: 0,
        timezone: "".to_string(),
        owner_id: "".to_string(),
        priority: 0,
        status: PeeringClaimStatus::Requested,
    }
}
