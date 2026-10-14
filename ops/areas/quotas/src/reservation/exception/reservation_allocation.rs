use super::*;

use acme_kit::text;

/// Where a reservation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ReservationAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl ReservationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for ReservationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a reservation allocation and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct ReservationAllocation {
    pub code: String,
    pub label: String,
    pub tags: Vec<String>,
    pub tenant_id: String,
    pub host: String,
    pub fingerprint: String,
    pub amount: f64,
    pub unit: String,
    pub limit: i64,
    pub cluster: String,
    pub status: ReservationAllocationStatus,
}

impl ReservationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            ReservationAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: ReservationAllocationStatus) -> bool {
        match self.status {
            ReservationAllocationStatus::Requested => {
                return next == ReservationAllocationStatus::Expired;
            }
            ReservationAllocationStatus::Granted => {
                match next {
                    ReservationAllocationStatus::Revoked | ReservationAllocationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            ReservationAllocationStatus::Revoked => {
                return next == ReservationAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: ReservationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "reservation allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn display_label(&self) -> String {
        format!("{} [{}]", self.code, self.label)
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        for candidate in self.tags.iter() {
            if text::normalize(candidate) == text::normalize(tag) {
                return true;
            }
        }
        false
    }

    pub fn projected_load(&self) -> f64 {
        ((self.limit as f64) * self.amount * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> ReservationAllocation {
        ReservationAllocation {
            code: self.code.clone(),
            label: self.label.clone(),
            tags: self.tags.clone(),
            tenant_id: self.tenant_id.clone(),
            host: self.host.clone(),
            fingerprint: self.fingerprint.clone(),
            amount: self.amount,
            unit: self.unit.clone(),
            limit: self.limit,
            cluster: self.cluster.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.unit)
    }
}

/// Starts a reservation allocation in status requested.
pub fn new_reservation_allocation(code: &str, label: &str) -> ReservationAllocation {
    ReservationAllocation {
        code: code.to_string(),
        label: label.to_string(),
        tags: Vec::new(),
        tenant_id: "".to_string(),
        host: "".to_string(),
        fingerprint: "".to_string(),
        amount: 0.0,
        unit: "".to_string(),
        limit: 0,
        cluster: "".to_string(),
        status: ReservationAllocationStatus::Requested,
    }
}
