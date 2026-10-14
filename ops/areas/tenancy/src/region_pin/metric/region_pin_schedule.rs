use super::*;

use acme_kit::text;

/// Where a region pin schedule is in its life cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum RegionPinScheduleStatus {
    #[default]
    Created,
    Validated,
    Published,
    Retired,
}

impl RegionPinScheduleStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Validated => "validated",
            Self::Published => "published",
            Self::Retired => "retired",
        }
    }
}

impl std::fmt::Display for RegionPinScheduleStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The record of a region pin schedule and its current status.
#[derive(Debug, Clone, PartialEq)]
pub struct RegionPinSchedule {
    pub code: String,
    pub name: String,
    pub grace_days: i64,
    pub capacity: i64,
    pub score: f64,
    pub pinned: bool,
    pub currency: String,
    pub owner_id: String,
    pub limit: i64,
    pub status: RegionPinScheduleStatus,
}

impl RegionPinSchedule {
    /// Reports whether the record can no longer change status.
    pub fn is_terminal(&self) -> bool {
        match self.status {
            RegionPinScheduleStatus::Retired => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn can_move_to(&self, next: RegionPinScheduleStatus) -> bool {
        match self.status {
            RegionPinScheduleStatus::Created => {
                return next == RegionPinScheduleStatus::Validated;
            }
            RegionPinScheduleStatus::Validated => {
                return next == RegionPinScheduleStatus::Published || next == RegionPinScheduleStatus::Retired;
            }
            RegionPinScheduleStatus::Published => {
                return next == RegionPinScheduleStatus::Retired;
            }
            _ => {}
        }
        false
    }

    pub fn move_to(&mut self, next: RegionPinScheduleStatus) -> Result<(), String> {
        if !self.can_move_to(next) {
            return Err(format!(
                "region pin schedule {} cannot move from {} to {}",
                self.code, self.status, next
            ));
        }
        self.status = next;
        Ok(())
    }

    pub fn label(&self) -> String {
        format!("{} [{}]", self.code, self.name)
    }

    pub fn net_exposure(&self) -> f64 {
        ((self.grace_days as f64 - (self.limit as f64)) * 100.0).round() / 100.0
    }

    pub fn copy(&self) -> RegionPinSchedule {
        RegionPinSchedule {
            code: self.code.clone(),
            name: self.name.clone(),
            grace_days: self.grace_days,
            capacity: self.capacity,
            score: self.score,
            pinned: self.pinned,
            currency: self.currency.clone(),
            owner_id: self.owner_id.clone(),
            limit: self.limit,
            status: self.status,
        }
    }

    pub fn group_key(&self) -> String {
        text::normalize(&self.name)
    }
}

/// Starts a region pin schedule in status created.
pub fn new_region_pin_schedule(code: &str, name: &str) -> RegionPinSchedule {
    RegionPinSchedule {
        code: code.to_string(),
        name: name.to_string(),
        grace_days: 0,
        capacity: 0,
        score: 0.0,
        pinned: false,
        currency: "".to_string(),
        owner_id: "".to_string(),
        limit: 0,
        status: RegionPinScheduleStatus::Created,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_in_created() {
        let item = new_region_pin_schedule("code-401", "name-34");
        assert_eq!(item.status, RegionPinScheduleStatus::Created, "status");
        assert!(!item.is_terminal(), "terminal");
    }

    #[test]
    fn moves_to_validated() {
        let mut item = new_region_pin_schedule("code-401", "name-34");
        if item.move_to(RegionPinScheduleStatus::Validated).is_err() {
            panic!("unexpected empty result");
        }
        assert_eq!(item.status, RegionPinScheduleStatus::Validated, "status");
    }

    #[test]
    fn rejects_moves_from_retired() {
        let mut item = new_region_pin_schedule("code-401", "name-34");
        item.status = RegionPinScheduleStatus::Retired;
        assert!(
            item.move_to(RegionPinScheduleStatus::Created).is_err(),
            "expected move to fail"
        );
    }
}
