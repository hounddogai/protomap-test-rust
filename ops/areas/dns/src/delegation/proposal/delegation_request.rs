use super::*;

use acme_kit::text;

/// Where a delegation request is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationRequestStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl DelegationRequestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for DelegationRequestStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation request as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationRequest {
    pub code: String,
    pub label: String,
    pub external_ref: String,
    pub floor_value: f64,
    pub window_days: i64,
    pub port: i64,
    pub attempts: i64,
    pub checksum: String,
    pub amount: f64,
    pub score: f64,
    pub cost: f64,
    pub status: DelegationRequestStatus,
}

impl DelegationRequest {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationRequestStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationRequestStatus) -> bool {
        match self.status {
            DelegationRequestStatus::Created => {
                return next == DelegationRequestStatus::Published || next == DelegationRequestStatus::Retired;
            }
            DelegationRequestStatus::Validated => {
                return next == DelegationRequestStatus::Published;
            }
            DelegationRequestStatus::Published => {
                return next == DelegationRequestStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: DelegationRequestStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation request {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} {}", "delegation request", self.code)
    }

    pub fn projected_volume(&self) -> f64 {
        ((self.attempts as f64 - self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationRequest {
        DelegationRequest {
            code: self.code.clone(),
            label: self.label.clone(),
            external_ref: self.external_ref.clone(),
            floor_value: self.floor_value,
            window_days: self.window_days,
            port: self.port,
            attempts: self.attempts,
            checksum: self.checksum.clone(),
            amount: self.amount,
            score: self.score,
            cost: self.cost,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.label)
    }
}

/// Starts a delegation request in status created.
pub fn new_delegation_request(code: &str, label: &str) -> DelegationRequest {
    DelegationRequest {
        code: code.to_string(),
        label: label.to_string(),
        external_ref: "".to_string(),
        floor_value: 0.0,
        window_days: 0,
        port: 0,
        attempts: 0,
        checksum: "".to_string(),
        amount: 0.0,
        score: 0.0,
        cost: 0.0,
        status: DelegationRequestStatus::Created,
    }
}
