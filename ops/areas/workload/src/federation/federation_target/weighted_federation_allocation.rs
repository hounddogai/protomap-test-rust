use super::*;

use acme_kit::text;

/// Where a weighted federation allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum WeightedFederationAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl WeightedFederationAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for WeightedFederationAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A weighted federation allocation. Its code is unique within a tenant.
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedFederationAllocation {
    pub code: String,
    pub name: String,
    pub weight: f64,
    pub cost: f64,
    pub scope: String,
    pub rotation_days: i64,
    pub updated_at: i64,
    pub audience: String,
    pub external_ref: String,
    pub status: WeightedFederationAllocationStatus,
}

impl WeightedFederationAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            WeightedFederationAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: WeightedFederationAllocationStatus) -> bool {
        match self.status {
            WeightedFederationAllocationStatus::Requested => {
                match next {
                    WeightedFederationAllocationStatus::Granted | WeightedFederationAllocationStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationAllocationStatus::Granted => {
                match next {
                    WeightedFederationAllocationStatus::Revoked | WeightedFederationAllocationStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            WeightedFederationAllocationStatus::Revoked => {
                match next {
                    WeightedFederationAllocationStatus::Expired => {
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

    pub fn move_to(&mut self, next: WeightedFederationAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "weighted federation allocation {} cannot move from {} to {}",
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
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_value(&self) -> f64 {
        ((self.weight - (self.updated_at as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> WeightedFederationAllocation {
        WeightedFederationAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            weight: self.weight,
            cost: self.cost,
            scope: self.scope.clone(),
            rotation_days: self.rotation_days,
            updated_at: self.updated_at,
            audience: self.audience.clone(),
            external_ref: self.external_ref.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.scope)
    }
}

/// Starts a weighted federation allocation in status requested.
pub fn new_weighted_federation_allocation(code: &str, name: &str) -> WeightedFederationAllocation {
    WeightedFederationAllocation {
        code: code.to_string(),
        name: name.to_string(),
        weight: 0.0,
        cost: 0.0,
        scope: "".to_string(),
        rotation_days: 0,
        updated_at: 0,
        audience: "".to_string(),
        external_ref: "".to_string(),
        status: WeightedFederationAllocationStatus::Requested,
    }
}
