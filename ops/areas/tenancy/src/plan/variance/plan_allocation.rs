use super::*;

use acme_kit::text;

/// Where a plan allocation is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum PlanAllocationStatus {
    #[default]
    Requested,
    Granted,
    Revoked,
    Expired,
}

impl PlanAllocationStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Requested => "requested",
            Self::Granted => "granted",
            Self::Revoked => "revoked",
            Self::Expired => "expired",
        }
    }
}

impl std::fmt::Display for PlanAllocationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One plan allocation, with what the planners need to act on it.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanAllocation {
    pub code: String,
    pub name: String,
    pub budget: f64,
    pub rotation_days: i64,
    pub threshold: f64,
    pub country: String,
    pub revision: i64,
    pub locale: String,
    pub status: PlanAllocationStatus,
}

impl PlanAllocation {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            PlanAllocationStatus::Expired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: PlanAllocationStatus) -> bool {
        match self.status {
            PlanAllocationStatus::Requested => {
                return next == PlanAllocationStatus::Granted;
            }
            PlanAllocationStatus::Granted => {
                return next == PlanAllocationStatus::Revoked || next == PlanAllocationStatus::Expired;
            }
            PlanAllocationStatus::Revoked => {
                return next == PlanAllocationStatus::Expired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: PlanAllocationStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "plan allocation {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        self.revision += 1;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} {}", "plan allocation", self.code)
    }

    pub fn weighted_load(&self) -> f64 {
        ((self.revision as f64 + (self.rotation_days as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> PlanAllocation {
        PlanAllocation {
            code: self.code.clone(),
            name: self.name.clone(),
            budget: self.budget,
            rotation_days: self.rotation_days,
            threshold: self.threshold,
            country: self.country.clone(),
            revision: self.revision,
            locale: self.locale.clone(),
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.country)
    }
}

/// Starts a plan allocation in status requested.
pub fn new_plan_allocation(code: &str, name: &str) -> PlanAllocation {
    PlanAllocation {
        code: code.to_string(),
        name: name.to_string(),
        budget: 0.0,
        rotation_days: 0,
        threshold: 0.0,
        country: "".to_string(),
        revision: 0,
        locale: "".to_string(),
        status: PlanAllocationStatus::Requested,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_requested() {
        let item = new_plan_allocation("code-671", "name-903");
        assert_eq!(item.status, PlanAllocationStatus::Requested, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_granted() {
        let mut item = new_plan_allocation("code-671", "name-903");
        if item.move_to(PlanAllocationStatus::Granted).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, PlanAllocationStatus::Granted, "status");
    }

    #[test]
    fn rejects_moves_from_expired() {
        let mut item = new_plan_allocation("code-671", "name-903");
        item.status = PlanAllocationStatus::Expired;
        assert!(
            item.move_to(PlanAllocationStatus::Requested).is_err(),
            "expected move to fail"
        );
    }
}
