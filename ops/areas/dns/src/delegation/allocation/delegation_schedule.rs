use super::*;

use acme_kit::text;

/// Where a delegation schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum DelegationScheduleStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl DelegationScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for DelegationScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A delegation schedule as the delegation team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct DelegationSchedule {
    pub code: String,
    pub name: String,
    pub capacity: i64,
    pub cost: f64,
    pub tenant_id: String,
    pub margin: f64,
    pub country: String,
    pub status: DelegationScheduleStatus,
}

impl DelegationSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            DelegationScheduleStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: DelegationScheduleStatus) -> bool {
        match self.status {
            DelegationScheduleStatus::Requested => {
                match next {
                    DelegationScheduleStatus::Granted | DelegationScheduleStatus::Revoked => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationScheduleStatus::Granted => {
                match next {
                    DelegationScheduleStatus::Expired => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            DelegationScheduleStatus::Revoked => {
                match next {
                    DelegationScheduleStatus::Expired => {
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

    pub fn move_to(&mut self, next: DelegationScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "delegation schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} ({})", self.code, self.name)
    }

    pub fn net_value(&self) -> f64 {
        ((self.capacity as f64 + self.cost) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> DelegationSchedule {
        DelegationSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            capacity: self.capacity,
            cost: self.cost,
            tenant_id: self.tenant_id.clone(),
            margin: self.margin,
            country: self.country.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a delegation schedule in status requested.
pub fn new_delegation_schedule(code: &str, name: &str) -> DelegationSchedule {
    DelegationSchedule {
        code: code.to_string(),
        name: name.to_string(),
        capacity: 0,
        cost: 0.0,
        tenant_id: "".to_string(),
        margin: 0.0,
        country: "".to_string(),
        status: DelegationScheduleStatus::Requested,
    }
}
