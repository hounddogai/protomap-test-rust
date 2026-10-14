use super::*;

use acme_kit::text;

/// Where a primary tenant schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PrimaryTenantScheduleStatus {
    #[default]
    Proposed,
    Approved,
    Rejected,
    Withdrawn,
}

impl PrimaryTenantScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Proposed => "proposed",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Withdrawn => "withdrawn",
        }
    }
}

impl std::fmt::Display for PrimaryTenantScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A primary tenant schedule as the tenant team keeps it.
#[derive(Debug, Clone, PartialEq)]
pub struct PrimaryTenantSchedule {
    pub code: String,
    pub name: String,
    pub phone: String,
    pub floor_value: f64,
    pub grace_days: i64,
    pub locale: String,
    pub cluster: String,
    pub weight: f64,
    pub region: String,
    pub currency: String,
    pub status: PrimaryTenantScheduleStatus,
}

impl PrimaryTenantSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PrimaryTenantScheduleStatus::Withdrawn => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PrimaryTenantScheduleStatus) -> bool {
        match self.status {
            PrimaryTenantScheduleStatus::Proposed => {
                return next == PrimaryTenantScheduleStatus::Rejected || next == PrimaryTenantScheduleStatus::Withdrawn;
            }
            PrimaryTenantScheduleStatus::Approved => {
                match next {
                    PrimaryTenantScheduleStatus::Rejected => {
                        return true;
                    }
                    _ => {}
                }
                return false;
            }
            PrimaryTenantScheduleStatus::Rejected => {
                match next {
                    PrimaryTenantScheduleStatus::Withdrawn => {
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

    pub fn move_to(&mut self, next: PrimaryTenantScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "primary tenant schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} / {}", self.code, self.name)
    }

    pub fn projected_exposure(&self) -> f64 {
        ((self.weight + (self.grace_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PrimaryTenantSchedule {
        PrimaryTenantSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            phone: self.phone.clone(),
            floor_value: self.floor_value,
            grace_days: self.grace_days,
            locale: self.locale.clone(),
            cluster: self.cluster.clone(),
            weight: self.weight,
            region: self.region.clone(),
            currency: self.currency.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.locale)
    }
}

/// Starts a primary tenant schedule in status proposed.
pub fn new_primary_tenant_schedule(code: &str, name: &str) -> PrimaryTenantSchedule {
    PrimaryTenantSchedule {
        code: code.to_string(),
        name: name.to_string(),
        phone: "".to_string(),
        floor_value: 0.0,
        grace_days: 0,
        locale: "".to_string(),
        cluster: "".to_string(),
        weight: 0.0,
        region: "".to_string(),
        currency: "".to_string(),
        status: PrimaryTenantScheduleStatus::Proposed,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_proposed() {
        let item = new_primary_tenant_schedule("code-813", "name-445");
        assert_eq!(item.status, PrimaryTenantScheduleStatus::Proposed, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_rejected() {
        let mut item = new_primary_tenant_schedule("code-813", "name-445");
        if item.move_to(PrimaryTenantScheduleStatus::Rejected).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PrimaryTenantScheduleStatus::Rejected, "status");
    }

    #[test]
    fn rejects_moves_from_withdrawn() {
        let mut item = new_primary_tenant_schedule("code-813", "name-445");
        item.status = PrimaryTenantScheduleStatus::Withdrawn;
        assert!(
            item.move_to(PrimaryTenantScheduleStatus::Proposed).is_err(),
            "expected move to fail"
        );
    }
}
